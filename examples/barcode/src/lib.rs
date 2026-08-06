//! Barcode / QR generation demo — component, hook, PNG export.

use arkit::prelude::*;

/// Full encode surface — same order as [`BarcodeFormat::ALL`].
const FORMATS: &[BarcodeFormat] = BarcodeFormat::ALL;

#[component]
pub fn BarcodePage() -> Element {
    let mut contents = use_signal(|| String::from("https://example.com/arkit"));
    let mut format_index = use_signal(|| 0_usize);
    let mut options = use_signal(|| BarcodeOptions::qr(220));
    let mut status = use_signal(|| {
        format!(
            "Ready · {} formats — tap chips to cycle symbologies",
            FORMATS.len()
        )
    });
    let mut base64_preview = use_signal(String::new);

    let code = use_barcode(contents, options);

    let format = FORMATS[format_index().min(FORMATS.len().saturating_sub(1))];
    let format_label = format.label().to_string();
    let phase_label = match code.phase() {
        BarcodePhase::Empty => "empty".to_string(),
        BarcodePhase::Encoding => "encoding (worker)…".to_string(),
        BarcodePhase::Ready(artifact) => format!(
            "ready · {}×{} · {}",
            artifact.bitmap.width(),
            artifact.bitmap.height(),
            artifact.bitmap.format().label()
        ),
        BarcodePhase::Error(error) => format!("error · {}", error.message()),
    };
    let base64_code = code.clone();
    let bytes_code = code.clone();
    let save_code = code;

    rsx! {
        scroll {
            width: "100%",
            height: "100%",
            scroll_bar: "off",
            column {
                width: "100%",
                padding_top: 48.0,
                padding_right: 20.0,
                padding_bottom: 40.0,
                padding_left: 20.0,
                align_items: "center",
                background_color: "#FFFAFAFA",

                text {
                    content: "Barcode".to_string(),
                    font_size: 28.0,
                    font_weight: 700_i32,
                    font_color: "#FF18181B",
                    line_height: 34.0,
                }
                text {
                    content: "arkit feature = barcode · encode · SVG preview · PNG export"
                        .to_string(),
                    font_size: 13.0,
                    font_color: "#FF71717A",
                    margin_top: 6.0,
                    line_height: 18.0,
                    text_align: "center",
                }

                column {
                    margin_top: 28.0,
                    width: "100%",
                    padding_top: 20.0,
                    padding_right: 16.0,
                    padding_bottom: 20.0,
                    padding_left: 16.0,
                    background_color: "#FFFFFFFF",
                    border_radius: 16.0,
                    align_items: "center",

                    text {
                        content: format_label,
                        font_size: 12.0,
                        font_weight: 600_i32,
                        font_color: "#FF71717A",
                        margin_bottom: 12.0,
                    }

                    Barcode {
                        contents: contents(),
                        format: format,
                        size: if format.is_matrix() { 220.0 } else { 280.0 },
                        height: if format.is_matrix() { None } else { Some(96.0) },
                        options: Some(options()),
                    }

                    text {
                        content: phase_label,
                        font_size: 11.0,
                        font_color: "#FFA1A1AA",
                        margin_top: 14.0,
                        text_align: "center",
                    }
                }

                column {
                    margin_top: 24.0,
                    width: "100%",
                    align_items: "start",
                    text {
                        content: "Contents".to_string(),
                        font_size: 13.0,
                        font_weight: 600_i32,
                        font_color: "#FF3F3F46",
                        margin_bottom: 8.0,
                    }
                    textinput {
                        width: "100%",
                        height: 44.0,
                        value: contents(),
                        placeholder: sample_placeholder(format).to_string(),
                        font_size: 14.0,
                        font_color: "#FF18181B",
                        background_color: "#FFFFFFFF",
                        border_radius: 10.0,
                        padding_left: 12.0,
                        padding_right: 12.0,
                        onchange: move |evt| {
                            contents.set(evt.string_value.clone());
                            status.set("Contents updated".to_string());
                            base64_preview.set(String::new());
                        },
                    }
                    text {
                        content: sample_hint(format).to_string(),
                        font_size: 11.0,
                        font_color: "#FFA1A1AA",
                        margin_top: 6.0,
                        line_height: 15.0,
                    }
                }

                column {
                    margin_top: 20.0,
                    width: "100%",
                    align_items: "start",
                    text {
                        content: format!("Format ({}/{})", format_index() + 1, FORMATS.len()),
                        font_size: 13.0,
                        font_weight: 600_i32,
                        font_color: "#FF3F3F46",
                        margin_bottom: 8.0,
                    }
                    // Full-width list rows — no flex wrap (unreliable hit targets on device).
                    column {
                        width: "100%",
                        align_items: "stretch",
                        for (index, item) in FORMATS.iter().copied().enumerate() {
                            {
                                let selected = format_index() == index;
                                let label = item.label().to_string();
                                rsx! {
                                    button {
                                        key: "{index}",
                                        width: "100%",
                                        height: 44.0,
                                        margin_bottom: 8.0,
                                        border_radius: 10.0,
                                        background_color: if selected {
                                            0xFF18181B_u32
                                        } else {
                                            0xFFFFFFFF_u32
                                        },
                                        font_color: if selected {
                                            0xFFFAFAFA_u32
                                        } else {
                                            0xFF18181B_u32
                                        },
                                        font_size: 14.0,
                                        font_weight: if selected { 600_i32 } else { 400_i32 },
                                        onclick: move |_| {
                                            format_index.set(index);
                                            let next = FORMATS[index];
                                            options.set(options_for(next));
                                            if contents().trim().is_empty()
                                                || is_sample_payload(&contents())
                                            {
                                                contents.set(sample_payload(next).to_string());
                                            }
                                            status.set(format!("Format → {}", next.label()));
                                            base64_preview.set(String::new());
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }

                column {
                    margin_top: 12.0,
                    width: "100%",
                    align_items: "stretch",
                    text {
                        content: "Export".to_string(),
                        font_size: 13.0,
                        font_weight: 600_i32,
                        font_color: "#FF3F3F46",
                        margin_bottom: 8.0,
                    }
                    button {
                        width: "100%",
                        height: 44.0,
                        margin_bottom: 8.0,
                        border_radius: 10.0,
                        background_color: "#FF2563EB",
                        font_color: "#FFFFFFFF",
                        font_size: 14.0,
                        onclick: move |_| {
                            status.set("Encoding PNG off UI thread…".to_string());
                            base64_code.base64_png_async(move |result| match result {
                                Ok(b64) => {
                                    let preview = if b64.len() > 48 {
                                        format!("{}… ({} chars)", &b64[..48], b64.len())
                                    } else {
                                        b64
                                    };
                                    base64_preview.set(preview);
                                    status.set("PNG base64 ready".to_string());
                                }
                                Err(error) => {
                                    status.set(format!("base64 failed: {}", error.message()));
                                }
                            });
                        },
                        "PNG → base64"
                    }
                    button {
                        width: "100%",
                        height: 44.0,
                        margin_bottom: 8.0,
                        border_radius: 10.0,
                        background_color: "#FF18181B",
                        font_color: "#FFFFFFFF",
                        font_size: 14.0,
                        onclick: move |_| {
                            status.set("Encoding PNG off UI thread…".to_string());
                            bytes_code.png_bytes_async(move |result| match result {
                                Ok(bytes) => {
                                    status.set(format!("PNG bytes · {} B", bytes.len()));
                                }
                                Err(error) => {
                                    status.set(format!("png failed: {}", error.message()));
                                }
                            });
                        },
                        "PNG bytes"
                    }
                    button {
                        width: "100%",
                        height: 44.0,
                        margin_bottom: 8.0,
                        border_radius: 10.0,
                        background_color: "#FF059669",
                        font_color: "#FFFFFFFF",
                        font_size: 14.0,
                        onclick: move |_| {
                            let path = std::env::temp_dir().join("arkit-barcode-demo.png");
                            status.set("Saving PNG off UI thread…".to_string());
                            save_code.save_png_async(path, move |result| match result {
                                Ok(saved) => {
                                    status.set(format!("saved {}", saved.display()));
                                }
                                Err(error) => {
                                    status.set(format!("save failed: {}", error.message()));
                                }
                            });
                        },
                        "Save PNG"
                    }
                }

                column {
                    margin_top: 24.0,
                    width: "100%",
                    padding_top: 16.0,
                    padding_right: 16.0,
                    padding_bottom: 16.0,
                    padding_left: 16.0,
                    background_color: "#FFFFFFFF",
                    border_radius: 12.0,
                    align_items: "start",
                    text {
                        content: "Pure encode_barcode".to_string(),
                        font_size: 13.0,
                        font_weight: 600_i32,
                        font_color: "#FF3F3F46",
                    }
                    text {
                        content: pure_encode_summary(&contents()),
                        font_size: 12.0,
                        font_color: "#FF52525B",
                        margin_top: 8.0,
                        line_height: 17.0,
                    }
                }

                text {
                    content: status(),
                    font_size: 12.0,
                    font_color: "#FF2563EB",
                    margin_top: 20.0,
                    text_align: "center",
                    line_height: 17.0,
                }
                if !base64_preview().is_empty() {
                    text {
                        content: base64_preview(),
                        font_size: 10.0,
                        font_color: "#FF71717A",
                        margin_top: 8.0,
                        text_align: "center",
                        line_height: 14.0,
                    }
                }
            }
        }
    }
}

fn options_for(format: BarcodeFormat) -> BarcodeOptions {
    match format {
        BarcodeFormat::QrCode => BarcodeOptions::qr(220),
        BarcodeFormat::DataMatrix | BarcodeFormat::Aztec => {
            BarcodeOptions::qr(220).with_format(format)
        }
        // PDF417 prefers a wide canvas.
        BarcodeFormat::Pdf417 => BarcodeOptions {
            format: BarcodeFormat::Pdf417,
            width: 320,
            height: 120,
            margin: 2,
            dark: 0xFF00_0000,
            light: 0xFFFF_FFFF,
            qr_ec_level: None,
        },
        BarcodeFormat::Ean13 | BarcodeFormat::Ean8 | BarcodeFormat::UpcA | BarcodeFormat::UpcE => {
            BarcodeOptions {
                format,
                width: 300,
                height: 112,
                margin: 2,
                dark: 0xFF00_0000,
                light: 0xFFFF_FFFF,
                qr_ec_level: None,
            }
        }
        BarcodeFormat::Code128
        | BarcodeFormat::Code93
        | BarcodeFormat::Code39
        | BarcodeFormat::Codabar
        | BarcodeFormat::Itf => BarcodeOptions::code128(300, 96).with_format(format),
    }
}

fn sample_payload(format: BarcodeFormat) -> &'static str {
    match format {
        BarcodeFormat::QrCode => "https://example.com/arkit",
        BarcodeFormat::DataMatrix => "ARKIT-DM-001",
        BarcodeFormat::Aztec => "ARKIT-AZTEC",
        BarcodeFormat::Pdf417 => "PDF417 sample · arkit barcode",
        BarcodeFormat::Code128 => "ARKIT-2026",
        BarcodeFormat::Code93 => "ARKIT93",
        BarcodeFormat::Code39 => "ARKIT-39",
        BarcodeFormat::Codabar => "A123456A",
        // Valid check-digit samples for retail symbologies.
        BarcodeFormat::Ean13 => "5901234123457",
        BarcodeFormat::Ean8 => "96385074",
        BarcodeFormat::UpcA => "042100005264",
        BarcodeFormat::UpcE => "01234565",
        BarcodeFormat::Itf => "123456789012",
    }
}

fn sample_placeholder(format: BarcodeFormat) -> &'static str {
    match format {
        BarcodeFormat::Ean13 => "12/13-digit EAN",
        BarcodeFormat::Ean8 => "7/8-digit EAN-8",
        BarcodeFormat::UpcA => "11/12-digit UPC-A",
        BarcodeFormat::UpcE => "UPC-E digits",
        BarcodeFormat::Itf => "Even-length digits",
        BarcodeFormat::Codabar => "A…D start/stop",
        BarcodeFormat::Code39 => "A-Z 0-9 -.$/+%",
        BarcodeFormat::Code93 | BarcodeFormat::Code128 => "Alphanumeric",
        BarcodeFormat::Pdf417 | BarcodeFormat::DataMatrix | BarcodeFormat::Aztec => "Text payload",
        BarcodeFormat::QrCode => "URL or text",
    }
}

fn sample_hint(format: BarcodeFormat) -> &'static str {
    match format {
        BarcodeFormat::QrCode => "Any UTF-8 text or URL. Default EC level is M.",
        BarcodeFormat::DataMatrix => "Compact 2D matrix; common in logistics / industry.",
        BarcodeFormat::Aztec => "2D matrix with bullseye finder; good for tickets.",
        BarcodeFormat::Pdf417 => "Stacked linear 2D; wider canvas in this demo.",
        BarcodeFormat::Code128 => "Full ASCII; tickets and logistics.",
        BarcodeFormat::Code93 => "Compact alphanumeric (Code 39 family).",
        BarcodeFormat::Code39 => "Uppercase A–Z, digits, and -.$/+% space.",
        BarcodeFormat::Codabar => "Digits + −$:/.+ with A–D start/stop (sample A…A).",
        BarcodeFormat::Ean13 => "Valid 12/13-digit GTIN (sample includes check digit).",
        BarcodeFormat::Ean8 => "Short retail code; sample is a valid EAN-8.",
        BarcodeFormat::UpcA => "US retail UPC-A; sample includes check digit.",
        BarcodeFormat::UpcE => "Zero-suppressed UPC-E form.",
        BarcodeFormat::Itf => "Interleaved 2 of 5 — even number of digits only.",
    }
}

fn is_sample_payload(value: &str) -> bool {
    FORMATS
        .iter()
        .any(|format| sample_payload(*format) == value)
}

fn pure_encode_summary(contents: &str) -> String {
    // Demo only: this path is synchronous. Production UI must use
    // use_barcode / Barcode (worker) or call encode_barcode off the UI thread.
    format!(
        "UI path is async (BarcodePhase::Encoding). Pure encode_barcode is for \
         tests/workers — sample payload len = {}.",
        contents.len()
    )
}
