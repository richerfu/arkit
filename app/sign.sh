#!/usr/bin/env bash
# 用 SDK 内置 OpenHarmony 官方 CA 给 hap 签名(设备/模拟器信任该链)。
#
# 一次性生成 app 密钥对 + 证书 + provisioning profile 缓存到
# ~/.ohos/arkit-signing/,然后对传入的 unsigned hap 执行 sign-app。
#
# 用法: ./sign.sh <unsigned.hap> <signed.hap>
set -euo pipefail

JAVA="/Applications/DevEco-Studio.app/Contents/jbr/Contents/Home/bin/java"
KEYTOOL="$(command -v keytool)"
SDK_LIB="/Applications/DevEco-Studio.app/Contents/sdk/default/openharmony/toolchains/lib"
HAP_SIGN_TOOL="$SDK_LIB/hap-sign-tool.jar"
# OpenHarmony 7.0+ 设备要求 hap 内 native 库带 ELF code signature。
BINARY_SIGN_TOOL="$SDK_LIB/binary-sign-tool.jar"
# 官方 keystore:密码 123456,内含受设备信任的
# "openharmony application ca" / "openharmony application root ca" /
# "openharmony application profile debug"。
OFFICIAL_P12="$SDK_LIB/OpenHarmony.p12"
PROFILE_DEBUG_PEM="$SDK_LIB/OpenHarmonyProfileDebug.pem"
WORK="$HOME/.ohos/arkit-signing"
BUNDLE="com.arkit.example"
PWD_PLAIN="123456"
# 注意:issuer/subject 字符串必须与设备 trust-cert-path 里的条目一致
# （keytool 导出顺序:C → O → OU → CN）,hap-sign-tool 做的是字符串比对。
# 模拟器(OpenHarmony 7.0)只信任 OpenHarmony Application CA 签发的
# "OpenHarmony Application Release" 作为 app 证书。
APP_CA_SUBJECT="C=CN, O=OpenHarmony, OU=OpenHarmony Team, CN=OpenHarmony Application CA"
APP_SUBJECT="C=CN, O=OpenHarmony, OU=OpenHarmony Team, CN=OpenHarmony Application Release"
# Debug profile 按 UDID 绑定设备。换设备/模拟器时用 `hdc shell bm get -u` 更新。
DEVICE_UDID="${DEVICE_UDID:-6F7004D23D437A26667B46CE40A1142A23FBF7E880DBB5DADF65758DB91DD14A}"

IN_FILE="${1:?usage: $0 <unsigned.hap> <signed.hap>}"
OUT_FILE="${2:?usage: $0 <unsigned.hap> <signed.hap>}"

ensure_materials() {
  [ -f "$WORK/app.p12" ] && [ -f "$WORK/app.cer" ] && [ -f "$WORK/app.p7b" ] && return 0

  echo ">> generating signing materials under $WORK"
  mkdir -p "$WORK"

  if [ ! -f "$WORK/app.p12" ] || [ ! -f "$WORK/app.cer" ]; then
    # App 密钥对(签名 hap 用)
    "$JAVA" -jar "$HAP_SIGN_TOOL" generate-keypair \
      -keyAlias appKey -keyPwd "$PWD_PLAIN" -keyAlg ECC -keySize NIST-P-256 \
      -keystoreFile "$WORK/app.p12" -keystorePwd "$PWD_PLAIN" >/dev/null

    # 从官方 keystore 导出证书链中间件(root ca / application ca)
    "$KEYTOOL" -exportcert -keystore "$OFFICIAL_P12" -storepass "$PWD_PLAIN" \
      -alias "openharmony application root ca" -rfc -file "$WORK/oh-root-ca.cer"
    "$KEYTOOL" -exportcert -keystore "$OFFICIAL_P12" -storepass "$PWD_PLAIN" \
      -alias "openharmony application ca" -rfc -file "$WORK/oh-app-ca.cer"

    # 官方 CA 为 app 密钥签发证书(证书链:app → app-ca → root-ca)
    "$JAVA" -jar "$HAP_SIGN_TOOL" generate-app-cert \
      -keyAlias appKey -keyPwd "$PWD_PLAIN" \
      -issuer "$APP_CA_SUBJECT" \
      -issuerKeyAlias "openharmony application ca" -issuerKeyPwd "$PWD_PLAIN" \
      -issuerKeystoreFile "$OFFICIAL_P12" -issuerKeystorePwd "$PWD_PLAIN" \
      -subject "$APP_SUBJECT" -validity 3650 -signAlg SHA256withECDSA \
      -keystoreFile "$WORK/app.p12" -keystorePwd "$PWD_PLAIN" \
      -outForm certChain -rootCaCertFile "$WORK/oh-root-ca.cer" \
      -subCaCertFile "$WORK/oh-app-ca.cer" -outFile "$WORK/app.cer" >/dev/null
  fi

  if [ ! -f "$WORK/app.p7b" ]; then
    PEM="$WORK/development-certificate.pem"
    awk 'BEGIN{p=0} /-----BEGIN CERTIFICATE-----/{p=1} p{print} /-----END CERTIFICATE-----/{p=0}' \
      "$WORK/app.cer" > "$PEM"
    PROFILE_JSON="$WORK/profile.json"
    NOW=$(date +%s)
    AFTER=$((NOW + 3650 * 86400))
    cat > "$PROFILE_JSON" <<EOF
{
  "version-name": "2.0.0",
  "version-code": 2,
  "uuid": "$(uuidgen | tr 'A-Z' 'a-z')",
  "validity": {
    "not-before": $((NOW - 86400)),
    "not-after": $AFTER
  },
  "type": "debug",
  "bundle-info": {
    "developer-id": "OpenHarmony",
    "development-certificate": $(python3 -c 'import json,sys; print(json.dumps(open(sys.argv[1]).read()))' "$PEM"),
    "bundle-name": "$BUNDLE",
    "apl": "normal",
    "app-feature": "hos_normal_app"
  },
  "acls": {
    "allowed-acls": ["ohos.permission.CAMERA"]
  },
  "permissions": {
    "restricted-permissions": ["ohos.permission.CAMERA"]
  },
  "debug-info": {
    "device-ids": ["$DEVICE_UDID"],
    "device-id-type": "udid"
  },
  "issuer": "$APP_CA_SUBJECT"
}
EOF
    # profile 用官方 profile 证书签名
    "$JAVA" -jar "$HAP_SIGN_TOOL" sign-profile \
      -mode localSign -keyAlias "openharmony application profile debug" -keyPwd "$PWD_PLAIN" \
      -profileCertFile "$PROFILE_DEBUG_PEM" \
      -inFile "$PROFILE_JSON" -signAlg SHA256withECDSA \
      -keystoreFile "$OFFICIAL_P12" -keystorePwd "$PWD_PLAIN" \
      -outFile "$WORK/app.p7b" >/dev/null
  fi
}

ensure_materials

# 1) 对 hap 内所有 native 库做 ELF code signature(OpenHarmony 7.0+ 安装
#    时逐 ELF 校验,跳过会报 verify code signature failed)。
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
unzip -q -o "$IN_FILE" "libs/*" -d "$STAGE"
for SO in "$STAGE"/libs/arm64-v8a/*.so; do
  [ -f "$SO" ] || continue
  echo ">> ELF code signing $(basename "$SO")"
  "$JAVA" -jar "$BINARY_SIGN_TOOL" sign \
    -mode localSign -keyAlias appKey -keyPwd "$PWD_PLAIN" \
    -appCertFile "$WORK/app.cer" -profileFile "$WORK/app.p7b" \
    -inFile "$SO" -signAlg SHA256withECDSA \
    -keystoreFile "$WORK/app.p12" -keystorePwd "$PWD_PLAIN" \
    -outFile "$SO" >/dev/null
done
# 回填签名后的 .so(在 STAGE 目录内以相对路径更新 zip)
WORKING="$STAGE/working.hap"
cp "$IN_FILE" "$WORKING"
(cd "$STAGE" && zip -q "$WORKING" libs/arm64-v8a/*.so)

# 2) hap 整体签名
echo ">> signing $(basename "$IN_FILE") -> $(basename "$OUT_FILE")"
"$JAVA" -jar "$HAP_SIGN_TOOL" sign-app \
  -mode localSign -keyAlias appKey -keyPwd "$PWD_PLAIN" \
  -appCertFile "$WORK/app.cer" -profileFile "$WORK/app.p7b" \
  -inFile "$WORKING" -signAlg SHA256withECDSA \
  -keystoreFile "$WORK/app.p12" -keystorePwd "$PWD_PLAIN" \
  -outFile "$OUT_FILE" -compatibleVersion 12 -signCode 1
echo ">> signed: $OUT_FILE"
