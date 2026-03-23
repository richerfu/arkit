use std::sync::Once;

use ohos_hilog_binding::{self, LogOptions};

static INIT_HILOG: Once = Once::new();

pub(crate) fn init_hilog() {
    INIT_HILOG.call_once(|| {
        ohos_hilog_binding::set_global_options(LogOptions {
            domain: 0x0000,
            tag: "arkit",
        });
        ohos_hilog_binding::info("arkit hilog initialized");
    });
}

pub(crate) fn info(message: impl AsRef<str>) {
    init_hilog();
    ohos_hilog_binding::info(message.as_ref());
}

pub(crate) fn error(message: impl AsRef<str>) {
    init_hilog();
    ohos_hilog_binding::error(message.as_ref());
}
