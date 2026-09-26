# Third-party notices

This inventory follows this repository's Cargo/npm lockfiles. Dependency licenses remain their authors' licenses; the project MIT license does not replace them. Original notice files available for the exact resolved versions are preserved under `third_party/licenses`; exact-version cache or notice gaps are called out below.

The lockfiles include development, optional and cross-platform packages. Review the actual shipped dependency set and any reciprocal-license obligations before distributing binaries. An unavailable notice is a release-review item for artifacts that contain that dependency, not evidence that it is unlicensed.

## Project artwork review

The Connections UI bundles the official Google Drive product logo from [Google's Drive branding guidance](https://developers.google.com/workspace/drive/api/guides/branding) (`apps/desktop/public/provider-logos/google-drive.png`) and the official GIPHY square icon linked by the [GIPHY developer site](https://developers.giphy.com/) ([source asset](https://developers.giphy.com/branch/master/icons/icon-192x192.png?v=22918c3c9ee5a9845590e3d909ab493c), `apps/desktop/public/provider-logos/giphy.png`). These marks remain the property of Google LLC and GIPHY, respectively. They are bundled locally for display in the provider connection UI; the app makes no runtime request for either logo. Google Drive usage is subject to Google's trademark and Drive API terms. GIPHY's API attribution requirements apply where GIPHY content is used; this provider icon does not replace content attribution.

The current app bundle artwork is `apps/desktop/src-tauri/icons/icon.png`. The current UI also references `apps/desktop/public/art/command-surface.png`, `apps/desktop/public/art/selection-brush.png`, and `apps/desktop/public/art/voice-ink.png`. `voice-ink.png` and `selection-brush.png` were generated with the built-in ImageGen from user-approved mockup/design direction in this turn; no new external art licenses are claimed for them. The two bundled font files under `apps/desktop/public/fonts/` are Cormorant Garamond version 4.001, Copyright 2015 The Cormorant Project Authors (github.com/CatharsisFonts/Cormorant), licensed under the SIL Open Font License 1.1. The license text is in `third_party/licenses/asset-font-cormorant-garamond-4.001/LICENSE`, copied from the upstream repository. The creator/source, ownership and redistribution permission for the remaining artwork and icon have not been verified in this inventory. Before publishing source or distributing a binary, the owner must confirm those rights and add attribution or a separate notice if the assets require one. Do not treat the files' presence in the repository or `scripts/public-files.json` as licensing evidence.

| Ecosystem | Package | Version | Declared license |
|---|---|---|---|
| Rust | adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| Rust | aead | 0.5.2 | MIT OR Apache-2.0 |
| Rust | aes | 0.8.4 | MIT OR Apache-2.0 |
| Rust | aes-gcm | 0.10.3 | Apache-2.0 OR MIT |
| Rust | ahash | 0.8.12 | MIT OR Apache-2.0 |
| Rust | aho-corasick | 1.1.4 | Unlicense OR MIT |
| Rust | alloc-no-stdlib | 2.0.4 | BSD-3-Clause |
| Rust | alloc-stdlib | 0.2.2 | BSD-3-Clause |
| Rust | android_system_properties | 0.1.5 | MIT/Apache-2.0 |
| Rust | anyhow | 1.0.103 | MIT OR Apache-2.0 |
| Rust | arbitrary | 1.4.2 | MIT OR Apache-2.0 |
| Rust | atk | 0.18.2 | MIT |
| Rust | atk-sys | 0.18.2 | MIT |
| Rust | atomic-waker | 1.1.2 | Apache-2.0 OR MIT |
| Rust | autocfg | 1.5.1 | Apache-2.0 OR MIT |
| Rust | axum | 0.8.9 | MIT |
| Rust | axum-core | 0.5.6 | MIT |
| Rust | base64 | 0.21.7 | MIT OR Apache-2.0 |
| Rust | base64 | 0.22.1 | MIT OR Apache-2.0 |
| Rust | bit-set | 0.8.0 | Apache-2.0 OR MIT |
| Rust | bit-vec | 0.8.0 | Apache-2.0 OR MIT |
| Rust | bitflags | 1.3.2 | MIT/Apache-2.0 |
| Rust | bitflags | 2.13.2 | MIT OR Apache-2.0 |
| Rust | block-buffer | 0.10.4 | MIT OR Apache-2.0 |
| Rust | block2 | 0.6.2 | MIT |
| Rust | brotli | 8.0.2 | BSD-3-Clause AND MIT |
| Rust | brotli-decompressor | 5.0.0 | BSD-3-Clause/MIT |
| Rust | bs58 | 0.5.1 | MIT/Apache-2.0 |
| Rust | bumpalo | 3.20.3 | MIT OR Apache-2.0 |
| Rust | bytemuck | 1.25.0 | Zlib OR Apache-2.0 OR MIT |
| Rust | byteorder | 1.5.0 | Unlicense OR MIT |
| Rust | bytes | 1.12.1 | MIT |
| Rust | cairo-rs | 0.18.5 | MIT |
| Rust | cairo-sys-rs | 0.18.2 | MIT |
| Rust | camino | 1.2.2 | MIT OR Apache-2.0 |
| Rust | cargo-platform | 0.1.9 | MIT OR Apache-2.0 |
| Rust | cargo_metadata | 0.19.2 | MIT |
| Rust | cargo_toml | 0.22.3 | Apache-2.0 OR MIT |
| Rust | cc | 1.4.5 | MIT OR Apache-2.0 |
| Rust | cesu8 | 1.1.0 | Apache-2.0/MIT |
| Rust | cfb | 0.7.3 | MIT |
| Rust | cfg-expr | 0.15.8 | MIT OR Apache-2.0 |
| Rust | cfg-if | 1.0.4 | MIT OR Apache-2.0 |
| Rust | cfg_aliases | 0.2.2 | MIT |
| Rust | chacha20 | 0.10.2 | MIT OR Apache-2.0 |
| Rust | chrono | 0.4.44 | MIT OR Apache-2.0 |
| Rust | cipher | 0.4.4 | MIT OR Apache-2.0 |
| Rust | combine | 4.6.7 | MIT |
| Rust | cookie | 0.18.1 | MIT OR Apache-2.0 |
| Rust | core-foundation | 0.10.1 | MIT OR Apache-2.0 |
| Rust | core-foundation-sys | 0.8.7 | MIT OR Apache-2.0 |
| Rust | core-graphics | 0.25.0 | MIT OR Apache-2.0 |
| Rust | core-graphics-types | 0.2.0 | MIT OR Apache-2.0 |
| Rust | cpufeatures | 0.2.17 | MIT OR Apache-2.0 |
| Rust | cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| Rust | crc32fast | 1.5.0 | MIT OR Apache-2.0 |
| Rust | crossbeam-channel | 0.5.15 | MIT OR Apache-2.0 |
| Rust | crossbeam-utils | 0.8.21 | MIT OR Apache-2.0 |
| Rust | crypto-common | 0.1.7 | MIT OR Apache-2.0 |
| Rust | cssparser | 0.36.0 | MPL-2.0 |
| Rust | cssparser-macros | 0.6.1 | MPL-2.0 |
| Rust | ctor | 0.8.0 | Apache-2.0 OR MIT |
| Rust | ctor-proc-macro | 0.0.7 | Apache-2.0 OR MIT |
| Rust | ctr | 0.9.2 | MIT OR Apache-2.0 |
| Rust | darling | 0.23.0 | MIT |
| Rust | darling_core | 0.23.0 | MIT |
| Rust | darling_macro | 0.23.0 | MIT |
| Rust | data-encoding | 2.11.0 | MIT |
| Rust | dbus | 0.9.11 | Apache-2.0/MIT |
| Rust | deranged | 0.5.8 | MIT OR Apache-2.0 |
| Rust | derive_arbitrary | 1.4.2 | MIT OR Apache-2.0 |
| Rust | derive_more | 2.1.1 | MIT |
| Rust | derive_more-impl | 2.1.1 | MIT |
| Rust | digest | 0.10.7 | MIT OR Apache-2.0 |
| Rust | dirs | 6.0.0 | MIT OR Apache-2.0 |
| Rust | dirs-sys | 0.5.0 | MIT OR Apache-2.0 |
| Rust | dispatch2 | 0.3.1 | Zlib OR Apache-2.0 OR MIT |
| Rust | displaydoc | 0.2.7 | MIT OR Apache-2.0 |
| Rust | dlopen2 | 0.8.2 | MIT |
| Rust | dlopen2_derive | 0.4.3 | MIT |
| Rust | dom_query | 0.27.0 | MIT |
| Rust | dpi | 0.1.2 | Apache-2.0 AND MIT |
| Rust | dtoa | 1.0.11 | MIT OR Apache-2.0 |
| Rust | dtoa-short | 0.3.5 | MPL-2.0 |
| Rust | transcribe-rs | 0.3.11 | MIT |
| Rust | ort | 2.0.0-rc.12 | MIT OR Apache-2.0 |
| Rust | ort-sys | 2.0.0-rc.12 | MIT OR Apache-2.0 |
| Rust | ndarray | 0.17.2 | MIT OR Apache-2.0 |
| Rust | rustfft | 6.4.1 | MIT OR Apache-2.0 |
| Rust | dtor | 0.3.0 | Apache-2.0 OR MIT |
| Rust | dtor-proc-macro | 0.0.6 | Apache-2.0 OR MIT |
| Rust | dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 |
| Rust | dyn-clone | 1.0.20 | MIT OR Apache-2.0 |
| Rust | embed-resource | 3.0.9 | MIT |
| Rust | embed_plist | 1.2.2 | MIT OR Apache-2.0 |
| Rust | equivalent | 1.0.2 | Apache-2.0 OR MIT |
| Rust | erased-serde | 0.4.10 | MIT OR Apache-2.0 |
| Rust | errno | 0.3.14 | MIT OR Apache-2.0 |
| Rust | fallible-iterator | 0.3.0 | MIT/Apache-2.0 |
| Rust | fallible-streaming-iterator | 0.1.9 | MIT/Apache-2.0 |
| Rust | fastrand | 2.4.1 | Apache-2.0 OR MIT |
| Rust | fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| Rust | field-offset | 0.3.6 | MIT OR Apache-2.0 |
| Rust | filetime | 0.2.29 | MIT/Apache-2.0 |
| Rust | find-msvc-tools | 0.1.12 | MIT OR Apache-2.0 |
| Rust | flate2 | 1.1.9 | MIT OR Apache-2.0 |
| Rust | fnv | 1.0.7 | Apache-2.0 / MIT |
| Rust | foldhash | 0.2.0 | Zlib |
| Rust | foreign-types | 0.5.0 | MIT/Apache-2.0 |
| Rust | foreign-types-macros | 0.2.3 | MIT/Apache-2.0 |
| Rust | foreign-types-shared | 0.3.1 | MIT/Apache-2.0 |
| Rust | form_urlencoded | 1.2.2 | MIT OR Apache-2.0 |
| Rust | futures-channel | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-core | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-executor | 0.3.32 | MIT OR Apache-2.0 |
| Rust | futures-io | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-macro | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-sink | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-task | 0.3.34 | MIT OR Apache-2.0 |
| Rust | futures-util | 0.3.34 | MIT OR Apache-2.0 |
| Rust | gdk | 0.18.2 | MIT |
| Rust | gdk-pixbuf | 0.18.5 | MIT |
| Rust | gdk-pixbuf-sys | 0.18.0 | MIT |
| Rust | gdk-sys | 0.18.2 | MIT |
| Rust | gdkwayland-sys | 0.18.2 | MIT |
| Rust | gdkx11 | 0.18.2 | MIT |
| Rust | gdkx11-sys | 0.18.2 | MIT |
| Rust | generic-array | 0.14.7 | MIT |
| Rust | gethostname | 1.1.0 | Apache-2.0 |
| Rust | getrandom | 0.2.17 | MIT OR Apache-2.0 |
| Rust | getrandom | 0.3.4 | MIT OR Apache-2.0 |
| Rust | getrandom | 0.4.3 | MIT OR Apache-2.0 |
| Rust | ghash | 0.5.1 | Apache-2.0 OR MIT |
| Rust | gio | 0.18.4 | MIT |
| Rust | gio-sys | 0.18.1 | MIT |
| Rust | glib | 0.18.5 | MIT |
| Rust | glib-macros | 0.18.5 | MIT |
| Rust | glib-sys | 0.18.1 | MIT |
| Rust | glob | 0.3.3 | MIT OR Apache-2.0 |
| Rust | global-hotkey | 0.7.0 | Apache-2.0 OR MIT |
| Rust | gobject-sys | 0.18.0 | MIT |
| Rust | gtk | 0.18.2 | MIT |
| Rust | gtk-sys | 0.18.2 | MIT |
| Rust | gtk3-macros | 0.18.2 | MIT |
| Rust | hashbrown | 0.12.3 | MIT OR Apache-2.0 |
| Rust | hashbrown | 0.14.5 | MIT OR Apache-2.0 |
| Rust | hashbrown | 0.17.1 | MIT OR Apache-2.0 |
| Rust | hashlink | 0.9.1 | MIT OR Apache-2.0 |
| Rust | heck | 0.4.1 | MIT OR Apache-2.0 |
| Rust | heck | 0.5.0 | MIT OR Apache-2.0 |
| Rust | hex | 0.4.3 | MIT OR Apache-2.0 |
| Rust | html5ever | 0.38.0 | MIT OR Apache-2.0 |
| Rust | http | 1.5.0 | MIT OR Apache-2.0 |
| Rust | http-body | 1.1.0 | MIT |
| Rust | http-body-util | 0.1.5 | MIT |
| Rust | httparse | 1.10.1 | MIT OR Apache-2.0 |
| Rust | httpdate | 1.0.3 | MIT OR Apache-2.0 |
| Rust | hyper | 1.11.1 | MIT |
| Rust | hyper-rustls | 0.27.9 | Apache-2.0 OR ISC OR MIT |
| Rust | hyper-util | 0.1.20 | MIT |
| Rust | iana-time-zone | 0.1.65 | MIT OR Apache-2.0 |
| Rust | iana-time-zone-haiku | 0.1.2 | MIT OR Apache-2.0 |
| Rust | ico | 0.5.0 | MIT |
| Rust | icu_collections | 2.3.0 | Unicode-3.0 |
| Rust | icu_locale_core | 2.3.0 | Unicode-3.0 |
| Rust | icu_normalizer | 2.3.0 | Unicode-3.0 |
| Rust | icu_normalizer_data | 2.3.0 | Unicode-3.0 |
| Rust | icu_properties | 2.3.0 | Unicode-3.0 |
| Rust | icu_properties_data | 2.3.0 | Unicode-3.0 |
| Rust | icu_provider | 2.3.1 | Unicode-3.0 |
| Rust | ident_case | 1.0.1 | MIT/Apache-2.0 |
| Rust | idna | 1.1.0 | MIT OR Apache-2.0 |
| Rust | idna_adapter | 1.2.2 | Apache-2.0 OR MIT |
| Rust | indexmap | 1.9.3 | Apache-2.0 OR MIT |
| Rust | indexmap | 2.14.0 | Apache-2.0 OR MIT |
| Rust | infer | 0.19.0 | MIT |
| Rust | inout | 0.1.4 | MIT OR Apache-2.0 |
| Rust | ipnet | 2.12.2 | MIT OR Apache-2.0 |
| Rust | itoa | 1.0.18 | MIT OR Apache-2.0 |
| Rust | javascriptcore-rs | 1.1.2 | MIT |
| Rust | javascriptcore-rs-sys | 1.1.1 | MIT |
| Rust | jni | 0.21.1 | MIT/Apache-2.0 |
| Rust | jni | 0.22.4 | MIT OR Apache-2.0 |
| Rust | jni-macros | 0.22.4 | MIT OR Apache-2.0 |
| Rust | jni-sys | 0.3.1 | MIT OR Apache-2.0 |
| Rust | jni-sys | 0.4.1 | MIT OR Apache-2.0 |
| Rust | jni-sys-macros | 0.4.1 | MIT OR Apache-2.0 |
| Rust | js-sys | 0.3.99 | MIT OR Apache-2.0 |
| Rust | json-patch | 3.0.1 | MIT/Apache-2.0 |
| Rust | jsonptr | 0.6.3 | MIT OR Apache-2.0 |
| Rust | keyboard-types | 0.7.0 | MIT OR Apache-2.0 |
| Rust | lazy_static | 1.5.0 | MIT OR Apache-2.0 |
| Rust | libappindicator | 0.9.0 | Apache-2.0 OR MIT |
| Rust | libappindicator-sys | 0.9.0 | Apache-2.0 OR MIT |
| Rust | libc | 0.2.189 | MIT OR Apache-2.0 |
| Rust | libdbus-sys | 0.2.7 | Apache-2.0/MIT |
| Rust | libloading | 0.7.4 | ISC |
| Rust | libredox | 0.1.24 | MIT |
| Rust | libsqlite3-sys | 0.30.1 | MIT |
| Rust | linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| Rust | litemap | 0.8.3 | Unicode-3.0 |
| Rust | lock_api | 0.4.14 | MIT OR Apache-2.0 |
| Rust | log | 0.4.34 | MIT OR Apache-2.0 |
| Rust | lru-slab | 0.1.2 | MIT OR Apache-2.0 OR Zlib |
| Rust | markup5ever | 0.38.0 | MIT OR Apache-2.0 |
| Rust | matchers | 0.2.0 | MIT |
| Rust | matchit | 0.8.4 | MIT AND BSD-3-Clause |
| Rust | memchr | 2.8.3 | Unlicense OR MIT |
| Rust | memoffset | 0.9.1 | MIT |
| Rust | mime | 0.3.17 | MIT OR Apache-2.0 |
| Rust | mime_guess | 2.0.5 | MIT |
| Rust | minisign-verify | 0.2.5 | MIT |
| Rust | miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| Rust | mio | 1.2.3 | MIT |
| Rust | muda | 0.19.2 | Apache-2.0 OR MIT |
| Rust | ndk | 0.9.0 | MIT OR Apache-2.0 |
| Rust | ndk-sys | 0.6.0+11769913 | MIT OR Apache-2.0 |
| Rust | new_debug_unreachable | 1.0.6 | MIT |
| Rust | nu-ansi-term | 0.50.3 | MIT |
| Rust | num-conv | 0.2.2 | MIT OR Apache-2.0 |
| Rust | num-traits | 0.2.19 | MIT OR Apache-2.0 |
| Rust | num_enum | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| Rust | num_enum_derive | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 |
| Rust | objc2 | 0.6.4 | MIT |
| Rust | objc2-app-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-cloud-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-data | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-foundation | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-graphics | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-image | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-location | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-core-text | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-encode | 4.1.0 | MIT |
| Rust | objc2-exception-helper | 0.1.1 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-foundation | 0.3.2 | MIT |
| Rust | objc2-io-surface | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-osa-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-quartz-core | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-ui-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-user-notifications | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | objc2-web-kit | 0.3.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | once_cell | 1.21.4 | MIT OR Apache-2.0 |
| Rust | opaque-debug | 0.3.1 | MIT OR Apache-2.0 |
| Rust | openssl-probe | 0.2.1 | MIT OR Apache-2.0 |
| Rust | option-ext | 0.2.0 | MPL-2.0 |
| Rust | osakit | 0.3.1 | MIT OR Apache-2.0 |
| Rust | pango | 0.18.3 | MIT |
| Rust | pango-sys | 0.18.0 | MIT |
| Rust | parking_lot | 0.12.5 | MIT OR Apache-2.0 |
| Rust | parking_lot_core | 0.9.12 | MIT OR Apache-2.0 |
| Rust | percent-encoding | 2.3.2 | MIT OR Apache-2.0 |
| Rust | phf | 0.13.1 | MIT |
| Rust | phf_codegen | 0.13.1 | MIT |
| Rust | phf_generator | 0.13.1 | MIT |
| Rust | phf_macros | 0.13.1 | MIT |
| Rust | phf_shared | 0.13.1 | MIT |
| Rust | pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| Rust | pkg-config | 0.3.33 | MIT OR Apache-2.0 |
| Rust | plist | 1.10.0 | MIT |
| Rust | png | 0.17.16 | MIT OR Apache-2.0 |
| Rust | png | 0.18.1 | MIT OR Apache-2.0 |
| Rust | polyval | 0.6.2 | Apache-2.0 OR MIT |
| Rust | potential_utf | 0.1.6 | Unicode-3.0 |
| Rust | powerfmt | 0.2.0 | MIT OR Apache-2.0 |
| Rust | ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 |
| Rust | precomputed-hash | 0.1.1 | MIT |
| Rust | proc-macro-crate | 1.3.1 | MIT OR Apache-2.0 |
| Rust | proc-macro-crate | 2.0.2 | MIT OR Apache-2.0 |
| Rust | proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 |
| Rust | proc-macro-error | 1.0.4 | MIT OR Apache-2.0 |
| Rust | proc-macro-error-attr | 1.0.4 | MIT OR Apache-2.0 |
| Rust | proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| Rust | quick-xml | 0.41.0 | MIT |
| Rust | quinn | 0.11.11 | MIT OR Apache-2.0 |
| Rust | quinn-proto | 0.11.17 | MIT OR Apache-2.0 |
| Rust | quinn-udp | 0.5.15 | MIT OR Apache-2.0 |
| Rust | quote | 1.0.47 | MIT OR Apache-2.0 |
| Rust | r-efi | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| Rust | r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| Rust | rand | 0.10.2 | MIT OR Apache-2.0 |
| Rust | rand | 0.8.8 | MIT OR Apache-2.0 |
| Rust | rand | 0.9.4 | MIT OR Apache-2.0 |
| Rust | rand_chacha | 0.3.1 | MIT OR Apache-2.0 |
| Rust | rand_chacha | 0.9.0 | MIT OR Apache-2.0 |
| Rust | rand_core | 0.10.1 | MIT OR Apache-2.0 |
| Rust | rand_core | 0.6.4 | MIT OR Apache-2.0 |
| Rust | rand_core | 0.9.5 | MIT OR Apache-2.0 |
| Rust | rand_pcg | 0.10.2 | MIT OR Apache-2.0 |
| Rust | raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib |
| Rust | redox_syscall | 0.5.18 | MIT |
| Rust | redox_users | 0.5.2 | MIT |
| Rust | ref-cast | 1.0.25 | MIT OR Apache-2.0 |
| Rust | ref-cast-impl | 1.0.25 | MIT OR Apache-2.0 |
| Rust | regex | 1.12.3 | MIT OR Apache-2.0 |
| Rust | regex-automata | 0.4.14 | MIT OR Apache-2.0 |
| Rust | regex-lite | 0.1.9 | MIT OR Apache-2.0 |
| Rust | regex-syntax | 0.8.10 | MIT OR Apache-2.0 |
| Rust | reqwest | 0.12.28 | MIT OR Apache-2.0 |
| Rust | reqwest | 0.13.3 | MIT OR Apache-2.0 |
| Rust | ring | 0.17.14 | Apache-2.0 AND ISC |
| Rust | rusqlite | 0.32.1 | MIT |
| Rust | rustc-hash | 2.1.3 | Apache-2.0 OR MIT |
| Rust | rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| Rust | rustix | 1.1.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| Rust | rustls | 0.23.44 | Apache-2.0 OR ISC OR MIT |
| Rust | rustls-native-certs | 0.8.3 | Apache-2.0 OR ISC OR MIT |
| Rust | rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 |
| Rust | rustls-platform-verifier | 0.7.0 | MIT OR Apache-2.0 |
| Rust | rustls-platform-verifier-android | 0.1.1 | MIT OR Apache-2.0 |
| Rust | rustls-webpki | 0.103.15 | ISC |
| Rust | rustversion | 1.0.22 | MIT OR Apache-2.0 |
| Rust | ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| Rust | same-file | 1.0.6 | Unlicense/MIT |
| Rust | schannel | 0.1.29 | MIT |
| Rust | schemars | 0.8.22 | MIT |
| Rust | schemars | 0.9.0 | MIT |
| Rust | schemars | 1.2.1 | MIT |
| Rust | schemars_derive | 0.8.22 | MIT |
| Rust | scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| Rust | security-framework | 3.7.0 | MIT OR Apache-2.0 |
| Rust | security-framework-sys | 2.17.0 | MIT OR Apache-2.0 |
| Rust | selectors | 0.36.1 | MPL-2.0 |
| Rust | semver | 1.0.28 | MIT OR Apache-2.0 |
| Rust | serde | 1.0.229 | MIT OR Apache-2.0 |
| Rust | serde-untagged | 0.1.9 | MIT OR Apache-2.0 |
| Rust | serde_core | 1.0.229 | MIT OR Apache-2.0 |
| Rust | serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| Rust | serde_derive_internals | 0.29.1 | MIT OR Apache-2.0 |
| Rust | serde_json | 1.0.151 | MIT OR Apache-2.0 |
| Rust | serde_path_to_error | 0.1.20 | MIT OR Apache-2.0 |
| Rust | serde_repr | 0.1.20 | MIT OR Apache-2.0 |
| Rust | serde_spanned | 0.6.9 | MIT OR Apache-2.0 |
| Rust | serde_spanned | 1.1.1 | MIT OR Apache-2.0 |
| Rust | serde_urlencoded | 0.7.1 | MIT/Apache-2.0 |
| Rust | serde_with | 3.21.0 | MIT OR Apache-2.0 |
| Rust | serde_with_macros | 3.21.0 | MIT OR Apache-2.0 |
| Rust | serialize-to-javascript | 0.1.2 | MIT OR Apache-2.0 |
| Rust | serialize-to-javascript-impl | 0.1.2 | MIT OR Apache-2.0 |
| Rust | servo_arc | 0.4.3 | MIT OR Apache-2.0 |
| Rust | sha1 | 0.10.6 | MIT OR Apache-2.0 |
| Rust | sha2 | 0.10.9 | MIT OR Apache-2.0 |
| Rust | sharded-slab | 0.1.7 | MIT |
| Rust | shlex | 2.0.1 | MIT OR Apache-2.0 |
| Rust | signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 |
| Rust | simd-adler32 | 0.3.9 | MIT |
| Rust | simd_cesu8 | 1.1.1 | Apache-2.0 OR MIT |
| Rust | simdutf8 | 0.1.5 | MIT OR Apache-2.0 |
| Rust | siphasher | 1.0.3 | MIT/Apache-2.0 |
| Rust | slab | 0.4.12 | MIT |
| Rust | smallvec | 1.16.1 | MIT OR Apache-2.0 |
| Rust | socket2 | 0.6.5 | MIT OR Apache-2.0 |
| Rust | softbuffer | 0.4.8 | MIT OR Apache-2.0 |
| Rust | soup3 | 0.5.0 | MIT |
| Rust | soup3-sys | 0.5.0 | MIT |
| Rust | stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| Rust | string_cache | 0.9.0 | MIT OR Apache-2.0 |
| Rust | string_cache_codegen | 0.6.1 | MIT OR Apache-2.0 |
| Rust | strsim | 0.11.1 | MIT |
| Rust | subtle | 2.6.1 | BSD-3-Clause |
| Rust | symphonia | 0.5.5 | MPL-2.0 |
| Rust | symphonia-bundle-flac | 0.5.5 | MPL-2.0 |
| Rust | symphonia-bundle-mp3 | 0.5.5 | MPL-2.0 |
| Rust | symphonia-codec-aac | 0.5.5 | MPL-2.0 |
| Rust | symphonia-codec-pcm | 0.5.5 | MPL-2.0 |
| Rust | symphonia-codec-vorbis | 0.5.5 | MPL-2.0 |
| Rust | symphonia-core | 0.5.5 | MPL-2.0 |
| Rust | symphonia-format-isomp4 | 0.5.5 | MPL-2.0 |
| Rust | symphonia-format-ogg | 0.5.5 | MPL-2.0 |
| Rust | symphonia-format-riff | 0.5.5 | MPL-2.0 |
| Rust | symphonia-metadata | 0.5.5 | MPL-2.0 |
| Rust | symphonia-utils-xiph | 0.5.5 | MPL-2.0 |
| Rust | swift-rs | 1.0.7 | MIT OR Apache-2.0 |
| Rust | syn | 1.0.109 | MIT OR Apache-2.0 |
| Rust | syn | 2.0.119 | MIT OR Apache-2.0 |
| Rust | syn | 3.0.5 | MIT OR Apache-2.0 |
| Rust | sync_wrapper | 1.0.2 | Apache-2.0 |
| Rust | synstructure | 0.13.2 | MIT |
| Rust | system-deps | 6.2.2 | MIT OR Apache-2.0 |
| Rust | tao | 0.35.2 | Apache-2.0 |
| Rust | tao-macros | 0.1.3 | MIT OR Apache-2.0 |
| Rust | tar | 0.4.46 | MIT OR Apache-2.0 |
| Rust | target-lexicon | 0.12.16 | Apache-2.0 WITH LLVM-exception |
| Rust | tauri | 2.11.2 | Apache-2.0 OR MIT |
| Rust | tauri-build | 2.6.2 | Apache-2.0 OR MIT |
| Rust | tauri-codegen | 2.6.2 | Apache-2.0 OR MIT |
| Rust | tauri-macros | 2.6.2 | Apache-2.0 OR MIT |
| Rust | tauri-plugin | 2.6.2 | Apache-2.0 OR MIT |
| Rust | tauri-plugin-global-shortcut | 2.3.1 | Apache-2.0 OR MIT |
| Rust | tauri-plugin-updater | 2.10.1 | Apache-2.0 OR MIT |
| Rust | tauri-runtime | 2.11.2 | Apache-2.0 OR MIT |
| Rust | tauri-runtime-wry | 2.11.2 | Apache-2.0 OR MIT |
| Rust | tauri-utils | 2.9.2 | Apache-2.0 OR MIT |
| Rust | tauri-winres | 0.3.6 | MIT |
| Rust | tempfile | 3.27.0 | MIT OR Apache-2.0 |
| Rust | tendril | 0.5.0 | MIT OR Apache-2.0 |
| Rust | thiserror | 1.0.69 | MIT OR Apache-2.0 |
| Rust | thiserror | 2.0.20 | MIT OR Apache-2.0 |
| Rust | thiserror-impl | 1.0.69 | MIT OR Apache-2.0 |
| Rust | thiserror-impl | 2.0.20 | MIT OR Apache-2.0 |
| Rust | thread_local | 1.1.9 | MIT OR Apache-2.0 |
| Rust | time | 0.3.55 | MIT OR Apache-2.0 |
| Rust | time-core | 0.1.9 | MIT OR Apache-2.0 |
| Rust | time-macros | 0.2.32 | MIT OR Apache-2.0 |
| Rust | tinystr | 0.8.4 | Unicode-3.0 |
| Rust | tinyvec | 1.13.2 | Zlib OR Apache-2.0 OR MIT |
| Rust | tinyvec_macros | 0.1.1 | MIT OR Apache-2.0 OR Zlib |
| Rust | tokio | 1.53.1 | MIT |
| Rust | tokio-macros | 2.7.2 | MIT |
| Rust | tokio-rustls | 0.26.5 | MIT OR Apache-2.0 |
| Rust | tokio-tungstenite | 0.29.0 | MIT |
| Rust | tokio-util | 0.7.19 | MIT |
| Rust | toml | 0.8.2 | MIT OR Apache-2.0 |
| Rust | toml | 0.9.12+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml_datetime | 0.6.3 | MIT OR Apache-2.0 |
| Rust | toml_datetime | 0.7.5+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml_datetime | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml_edit | 0.19.15 | MIT OR Apache-2.0 |
| Rust | toml_edit | 0.20.2 | MIT OR Apache-2.0 |
| Rust | toml_edit | 0.25.11+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml_parser | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | toml_writer | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 |
| Rust | tower | 0.5.3 | MIT |
| Rust | tower-http | 0.6.11 | MIT |
| Rust | tower-layer | 0.3.3 | MIT |
| Rust | tower-service | 0.3.3 | MIT |
| Rust | tracing | 0.1.44 | MIT |
| Rust | tracing-attributes | 0.1.31 | MIT |
| Rust | tracing-core | 0.1.36 | MIT |
| Rust | tracing-log | 0.2.0 | MIT |
| Rust | tracing-subscriber | 0.3.23 | MIT |
| Rust | tray-icon | 0.23.1 | MIT OR Apache-2.0 |
| Rust | try-lock | 0.2.5 | MIT |
| Rust | tungstenite | 0.29.0 | MIT OR Apache-2.0 |
| Rust | typeid | 1.0.3 | MIT OR Apache-2.0 |
| Rust | typenum | 1.20.1 | MIT OR Apache-2.0 |
| Rust | unic-char-property | 0.9.0 | MIT/Apache-2.0 |
| Rust | unic-char-range | 0.9.0 | MIT/Apache-2.0 |
| Rust | unic-common | 0.9.0 | MIT/Apache-2.0 |
| Rust | unic-ucd-ident | 0.9.0 | MIT/Apache-2.0 |
| Rust | unic-ucd-version | 0.9.0 | MIT/Apache-2.0 |
| Rust | unicase | 2.9.0 | MIT OR Apache-2.0 |
| Rust | unicode-ident | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| Rust | unicode-segmentation | 1.13.2 | MIT OR Apache-2.0 |
| Rust | universal-hash | 0.5.1 | MIT OR Apache-2.0 |
| Rust | untrusted | 0.9.0 | ISC |
| Rust | url | 2.5.8 | MIT OR Apache-2.0 |
| Rust | urlencoding | 2.1.3 | MIT |
| Rust | urlpattern | 0.3.0 | MIT |
| Rust | utf-8 | 0.7.6 | MIT OR Apache-2.0 |
| Rust | utf8_iter | 1.0.4 | Apache-2.0 OR MIT |
| Rust | uuid | 1.26.1 | Apache-2.0 OR MIT |
| Rust | valuable | 0.1.1 | MIT |
| Rust | vcpkg | 0.2.15 | MIT/Apache-2.0 |
| Rust | version-compare | 0.2.1 | MIT |
| Rust | version_check | 0.9.5 | MIT/Apache-2.0 |
| Rust | vswhom | 0.1.0 | MIT |
| Rust | vswhom-sys | 0.1.3 | MIT |
| Rust | walkdir | 2.5.0 | Unlicense/MIT |
| Rust | want | 0.3.1 | MIT |
| Rust | wasi | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| Rust | wasip2 | 1.0.3+wasi-0.2.9 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| Rust | wasm-bindgen | 0.2.122 | MIT OR Apache-2.0 |
| Rust | wasm-bindgen-futures | 0.4.72 | MIT OR Apache-2.0 |
| Rust | wasm-bindgen-macro | 0.2.122 | MIT OR Apache-2.0 |
| Rust | wasm-bindgen-macro-support | 0.2.122 | MIT OR Apache-2.0 |
| Rust | wasm-bindgen-shared | 0.2.122 | MIT OR Apache-2.0 |
| Rust | wasm-streams | 0.4.2 | MIT OR Apache-2.0 |
| Rust | wasm-streams | 0.5.0 | MIT OR Apache-2.0 |
| Rust | web-sys | 0.3.99 | MIT OR Apache-2.0 |
| Rust | web-time | 1.1.0 | MIT OR Apache-2.0 |
| Rust | web_atoms | 0.2.4 | MIT OR Apache-2.0 |
| Rust | webkit2gtk | 2.0.2 | MIT |
| Rust | webkit2gtk-sys | 2.0.2 | MIT |
| Rust | webpki-root-certs | 1.0.7 | CDLA-Permissive-2.0 |
| Rust | webpki-roots | 1.0.9 | CDLA-Permissive-2.0 |
| Rust | webview2-com | 0.38.2 | MIT |
| Rust | webview2-com-macros | 0.8.1 | MIT |
| Rust | webview2-com-sys | 0.38.2 | MIT |
| Rust | whisper-rs | 0.16.0 | Unlicense |
| Rust | whisper-rs-sys | 0.15.0 | Unlicense |
| Rust | winapi | 0.3.9 | MIT/Apache-2.0 |
| Rust | winapi-i686-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| Rust | winapi-util | 0.1.11 | Unlicense OR MIT |
| Rust | winapi-x86_64-pc-windows-gnu | 0.4.0 | MIT/Apache-2.0 |
| Rust | window-vibrancy | 0.6.0 | Apache-2.0 OR MIT |
| Rust | windows | 0.61.3 | MIT OR Apache-2.0 |
| Rust | windows-collections | 0.2.0 | MIT OR Apache-2.0 |
| Rust | windows-core | 0.61.2 | MIT OR Apache-2.0 |
| Rust | windows-future | 0.2.1 | MIT OR Apache-2.0 |
| Rust | windows-implement | 0.60.2 | MIT OR Apache-2.0 |
| Rust | windows-interface | 0.59.3 | MIT OR Apache-2.0 |
| Rust | windows-link | 0.1.3 | MIT OR Apache-2.0 |
| Rust | windows-link | 0.2.1 | MIT OR Apache-2.0 |
| Rust | windows-numerics | 0.2.0 | MIT OR Apache-2.0 |
| Rust | windows-result | 0.3.4 | MIT OR Apache-2.0 |
| Rust | windows-strings | 0.4.2 | MIT OR Apache-2.0 |
| Rust | windows-sys | 0.45.0 | MIT OR Apache-2.0 |
| Rust | windows-sys | 0.52.0 | MIT OR Apache-2.0 |
| Rust | windows-sys | 0.59.0 | MIT OR Apache-2.0 |
| Rust | windows-sys | 0.60.2 | MIT OR Apache-2.0 |
| Rust | windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| Rust | windows-targets | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows-targets | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows-targets | 0.53.5 | MIT OR Apache-2.0 |
| Rust | windows-threading | 0.1.0 | MIT OR Apache-2.0 |
| Rust | windows-version | 0.1.7 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_aarch64_msvc | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_i686_gnu | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_i686_gnu | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_i686_gnu | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_i686_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_i686_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_i686_msvc | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_i686_msvc | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_i686_msvc | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnu | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnu | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnu | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnullvm | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnullvm | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_gnullvm | 0.53.1 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_msvc | 0.42.2 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 |
| Rust | windows_x86_64_msvc | 0.53.1 | MIT OR Apache-2.0 |
| Rust | winnow | 0.5.40 | MIT |
| Rust | winnow | 0.7.15 | MIT |
| Rust | winnow | 1.0.3 | MIT |
| Rust | winreg | 0.55.0 | MIT |
| Rust | wit-bindgen | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| Rust | writeable | 0.6.4 | Unicode-3.0 |
| Rust | wry | 0.55.1 | Apache-2.0 OR MIT |
| Rust | x11 | 2.21.0 | MIT |
| Rust | x11-dl | 2.21.0 | MIT |
| Rust | x11rb | 0.13.2 | MIT OR Apache-2.0 |
| Rust | x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 |
| Rust | xattr | 1.6.1 | MIT OR Apache-2.0 |
| Rust | xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib |
| Rust | yoke | 0.8.3 | Unicode-3.0 |
| Rust | yoke-derive | 0.8.2 | Unicode-3.0 |
| Rust | zerocopy | 0.8.57 | BSD-2-Clause OR Apache-2.0 OR MIT |
| Rust | zerocopy-derive | 0.8.57 | Unresolved; review before distribution |
| Rust | zerofrom | 0.1.8 | Unicode-3.0 |
| Rust | zerofrom-derive | 0.1.7 | Unicode-3.0 |
| Rust | zeroize | 1.9.0 | Apache-2.0 OR MIT |
| Rust | zerotrie | 0.2.5 | Unicode-3.0 |
| Rust | zerovec | 0.11.8 | Unicode-3.0 |
| Rust | zerovec-derive | 0.11.6 | Unicode-3.0 |
| Rust | zip | 4.6.1 | MIT |
| Rust | zmij | 1.0.23 | MIT |
| npm | @esbuild/aix-ppc64 | 0.25.12 | MIT |
| npm | @esbuild/android-arm | 0.25.12 | MIT |
| npm | @esbuild/android-arm64 | 0.25.12 | MIT |
| npm | @esbuild/android-x64 | 0.25.12 | MIT |
| npm | @esbuild/darwin-arm64 | 0.25.12 | MIT |
| npm | @esbuild/darwin-x64 | 0.25.12 | MIT |
| npm | @esbuild/freebsd-arm64 | 0.25.12 | MIT |
| npm | @esbuild/freebsd-x64 | 0.25.12 | MIT |
| npm | @esbuild/linux-arm | 0.25.12 | MIT |
| npm | @esbuild/linux-arm64 | 0.25.12 | MIT |
| npm | @esbuild/linux-ia32 | 0.25.12 | MIT |
| npm | @esbuild/linux-loong64 | 0.25.12 | MIT |
| npm | @esbuild/linux-mips64el | 0.25.12 | MIT |
| npm | @esbuild/linux-ppc64 | 0.25.12 | MIT |
| npm | @esbuild/linux-riscv64 | 0.25.12 | MIT |
| npm | @esbuild/linux-s390x | 0.25.12 | MIT |
| npm | @esbuild/linux-x64 | 0.25.12 | MIT |
| npm | @esbuild/netbsd-arm64 | 0.25.12 | MIT |
| npm | @esbuild/netbsd-x64 | 0.25.12 | MIT |
| npm | @esbuild/openbsd-arm64 | 0.25.12 | MIT |
| npm | @esbuild/openbsd-x64 | 0.25.12 | MIT |
| npm | @esbuild/openharmony-arm64 | 0.25.12 | MIT |
| npm | @esbuild/sunos-x64 | 0.25.12 | MIT |
| npm | @esbuild/win32-arm64 | 0.25.12 | MIT |
| npm | @esbuild/win32-ia32 | 0.25.12 | MIT |
| npm | @esbuild/win32-x64 | 0.25.12 | MIT |
| npm | @jridgewell/gen-mapping | 0.3.13 | MIT |
| npm | @jridgewell/remapping | 2.3.5 | MIT |
| npm | @jridgewell/resolve-uri | 3.1.2 | MIT |
| npm | @jridgewell/sourcemap-codec | 1.6.0 | MIT |
| npm | @jridgewell/trace-mapping | 0.3.31 | MIT |
| npm | @lucide/svelte | 1.42.0 | ISC |
| npm | @napi-rs/lzma-linux-x64-gnu | 1.5.1 | MIT |
| npm | @rollup/rollup-android-arm-eabi | 4.63.1 | MIT |
| npm | @rollup/rollup-android-arm64 | 4.63.1 | MIT |
| npm | @rollup/rollup-darwin-arm64 | 4.63.1 | MIT |
| npm | @rollup/rollup-darwin-x64 | 4.63.1 | MIT |
| npm | @rollup/rollup-freebsd-arm64 | 4.63.1 | MIT |
| npm | @rollup/rollup-freebsd-x64 | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-arm-gnueabihf | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-arm-musleabihf | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-arm64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-arm64-musl | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-loong64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-loong64-musl | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-ppc64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-ppc64-musl | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-riscv64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-riscv64-musl | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-s390x-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-x64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-linux-x64-musl | 4.63.1 | MIT |
| npm | @rollup/rollup-openbsd-x64 | 4.63.1 | MIT |
| npm | @rollup/rollup-openharmony-arm64 | 4.63.1 | MIT |
| npm | @rollup/rollup-win32-arm64-msvc | 4.63.1 | MIT |
| npm | @rollup/rollup-win32-ia32-msvc | 4.63.1 | MIT |
| npm | @rollup/rollup-win32-x64-gnu | 4.63.1 | MIT |
| npm | @rollup/rollup-win32-x64-msvc | 4.63.1 | MIT |
| npm | @standard-schema/spec | 1.1.0 | MIT |
| npm | @sveltejs/acorn-typescript | 1.0.13 | MIT |
| npm | @sveltejs/load-config | 0.2.3 | MIT |
| npm | @sveltejs/vite-plugin-svelte | 5.1.1 | MIT |
| npm | @sveltejs/vite-plugin-svelte-inspector | 4.0.1 | MIT |
| npm | @tauri-apps/api | 2.11.1 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-darwin-arm64 | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-darwin-x64 | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-arm-gnueabihf | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-arm64-gnu | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-arm64-musl | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-riscv64-gnu | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-x64-gnu | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-linux-x64-musl | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-win32-arm64-msvc | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-win32-ia32-msvc | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/cli-win32-x64-msvc | 2.11.4 | Apache-2.0 OR MIT |
| npm | @tauri-apps/plugin-updater | 2.10.1 | MIT OR Apache-2.0 |
| npm | @types/chai | 5.2.3 | MIT |
| npm | @types/deep-eql | 4.0.2 | MIT |
| npm | @types/estree | 1.0.9 | MIT |
| npm | @types/node | 25.9.5 | MIT |
| npm | @vitest/expect | 4.1.11 | MIT |
| npm | @vitest/mocker | 4.1.11 | MIT |
| npm | @vitest/pretty-format | 4.1.11 | MIT |
| npm | @vitest/runner | 4.1.11 | MIT |
| npm | @vitest/snapshot | 4.1.11 | MIT |
| npm | @vitest/spy | 4.1.11 | MIT |
| npm | @vitest/utils | 4.1.11 | MIT |
| npm | acorn | 8.18.0 | MIT |
| npm | aria-query | 5.3.1 | Apache-2.0 |
| npm | assertion-error | 2.0.1 | MIT |
| npm | axobject-query | 4.1.0 | Apache-2.0 |
| npm | chai | 6.2.2 | MIT |
| npm | chokidar | 4.0.3 | MIT |
| npm | clsx | 2.1.1 | MIT |
| npm | convert-source-map | 2.0.0 | MIT |
| npm | debug | 4.4.3 | MIT |
| npm | deepmerge | 4.3.1 | MIT |
| npm | devalue | 5.9.2 | MIT |
| npm | es-module-lexer | 2.3.2 | MIT |
| npm | esbuild | 0.25.12 | MIT |
| npm | esm-env | 1.2.2 | MIT |
| npm | esrap | 2.3.7 | MIT |
| npm | estree-walker | 3.0.3 | MIT |
| npm | expect-type | 1.4.0 | Apache-2.0 |
| npm | fdir | 6.5.0 | MIT |
| npm | fsevents | 2.3.3 | MIT |
| npm | is-reference | 3.0.3 | MIT |
| npm | kleur | 4.1.5 | MIT |
| npm | locate-character | 3.0.0 | MIT |
| npm | magic-string | 0.30.21 | MIT |
| npm | mri | 1.2.0 | MIT |
| npm | ms | 2.1.3 | MIT |
| npm | nanoid | 3.3.18 | MIT |
| npm | obug | 2.2.1 | MIT |
| npm | pathe | 2.0.3 | MIT |
| npm | picocolors | 1.1.1 | ISC |
| npm | picomatch | 4.0.7 | MIT |
| npm | postcss | 8.5.28 | MIT |
| npm | readdirp | 4.1.2 | MIT |
| npm | rollup | 4.63.1 | MIT |
| npm | sade | 1.8.1 | MIT |
| npm | siginfo | 2.0.0 | ISC |
| npm | source-map-js | 1.2.1 | BSD-3-Clause |
| npm | stackback | 0.0.2 | MIT |
| npm | std-env | 4.2.0 | MIT |
| npm | svelte | 5.57.0 | MIT |
| npm | svelte-check | 4.7.6 | MIT |
| npm | tinybench | 2.9.0 | MIT |
| npm | tinyexec | 1.3.1 | MIT |
| npm | tinyglobby | 0.2.17 | MIT |
| npm | tinyrainbow | 3.1.1 | MIT |
| npm | typescript | 5.6.3 | Apache-2.0 |
| npm | undici-types | 7.24.6 | MIT |
| npm | vite | 6.4.3 | MIT |
| npm | vitefu | 1.1.3 | MIT |
| npm | vitest | 4.1.11 | MIT |
| npm | why-is-node-running | 2.3.0 | MIT |
| npm | zimmerframe | 1.1.5 | MIT |

## Notices requiring review

- The public local speech path adds `transcribe-rs 0.3.11`, `ort 2.0.0-rc.12`, and `ort-sys 2.0.0-rc.12` to the locked native graph. Preserve their exact MIT OR Apache-2.0 notices from the Cargo cache before distributing a binary. The ONNX Runtime archive is a build-time dependency and must not become an untracked runtime download.
- The reviewed NVIDIA Parakeet TDT 0.6B v3 ONNX export is a separate model resource, not a Rust dependency. Its upstream model cards declare CC BY 4.0; see [docs/LOCAL_MODELS.md](docs/LOCAL_MODELS.md) for the pinned commit, component URLs, sizes and upstream hash metadata. Review attribution and redistribution obligations before enabling an automated model bundle.
- The locked `whisper-rs 0.16.0` and `whisper-rs-sys 0.15.0` packages declare Unlicense in their local Cargo manifests. The exact `whisper-rs` package cache contains `LICENSE`; `whisper-rs-sys` includes the upstream `whisper.cpp/LICENSE` in its crate package manifest but no top-level license file was present in the inspected local cache. The package source is [whisper-rs on Codeberg](https://codeberg.org/tazz4843/whisper-rs); preserve the exact package/upstream license text, including the declared `whisper.cpp/LICENSE` path, in release notices before distributing a binary. This records package metadata and source locations, not a separate legal clearance of the vendored C/C++ source.
- The locked Symphonia family (`symphonia` plus its 11 enabled feature crates, all `0.5.5`) declares MPL-2.0 in the local Cargo manifests. The authoritative [Symphonia `rel-0.5` license](https://github.com/pdeljanov/Symphonia/blob/rel-0.5/LICENSE) is the upstream license source. No exact-version Symphonia notice is currently copied under `third_party/licenses`; preserve the applicable notice and review MPL-2.0 file-level obligations before distributing a binary.
- The native catalog can download the six Whisper artifacts and the five-file Parakeet export at pinned URLs. Their upstream-declared metadata, exact hashes, bundled-vs-user-downloaded distinction and unresolved provenance/redistribution questions are recorded in [docs/MODEL_LICENSES.md](docs/MODEL_LICENSES.md). Hash verification proves byte identity only; it does not establish ownership, permission or legal clearance.

- Rust alloc-stdlib 0.2.2
- Rust block2 0.6.2
- Rust dispatch2 0.3.1
- Rust dlopen2 0.8.2
- Rust dlopen2_derive 0.4.3
- Rust jni 0.22.4
- Rust jni-macros 0.22.4
- Rust jni-sys-macros 0.4.1
- Rust libappindicator-sys 0.9.0
- Rust ndk 0.9.0
- Rust ndk-sys 0.6.0+11769913
- Rust objc2 0.6.4
- Rust objc2-app-kit 0.3.2
- Rust objc2-cloud-kit 0.3.2
- Rust objc2-core-data 0.3.2
- Rust objc2-core-foundation 0.3.2
- Rust objc2-core-graphics 0.3.2
- Rust objc2-core-image 0.3.2
- Rust objc2-core-location 0.3.2
- Rust objc2-core-text 0.3.2
- Rust objc2-encode 4.1.0
- Rust objc2-exception-helper 0.1.1
- Rust objc2-foundation 0.3.2
- Rust objc2-io-surface 0.3.2
- Rust objc2-osa-kit 0.3.2
- Rust objc2-quartz-core 0.3.2
- Rust objc2-ui-kit 0.3.2
- Rust objc2-user-notifications 0.3.2
- Rust objc2-web-kit 0.3.2
- Rust r-efi 5.3.0
- Rust r-efi 6.0.0
- Rust rustls-platform-verifier-android 0.1.1
- Rust selectors 0.36.1
- Rust tao-macros 0.1.3
- Rust tauri-plugin 2.6.2
- Rust unic-char-property 0.9.0
- Rust unic-char-range 0.9.0
- Rust unic-common 0.9.0
- Rust unic-ucd-ident 0.9.0
- Rust unic-ucd-version 0.9.0
- Rust valuable 0.1.1
- Rust webview2-com 0.38.2
- Rust webview2-com-macros 0.8.1
- Rust webview2-com-sys 0.38.2
- Rust winapi-i686-pc-windows-gnu 0.4.0
- Rust winapi-x86_64-pc-windows-gnu 0.4.0
- Rust zerocopy-derive 0.8.57
- npm @esbuild/aix-ppc64 0.25.12
- npm @esbuild/android-arm 0.25.12
- npm @esbuild/android-arm64 0.25.12
- npm @esbuild/android-x64 0.25.12
- npm @esbuild/darwin-arm64 0.25.12
- npm @esbuild/darwin-x64 0.25.12
- npm @esbuild/freebsd-arm64 0.25.12
- npm @esbuild/freebsd-x64 0.25.12
- npm @esbuild/linux-arm 0.25.12
- npm @esbuild/linux-arm64 0.25.12
- npm @esbuild/linux-ia32 0.25.12
- npm @esbuild/linux-loong64 0.25.12
- npm @esbuild/linux-mips64el 0.25.12
- npm @esbuild/linux-ppc64 0.25.12
- npm @esbuild/linux-riscv64 0.25.12
- npm @esbuild/linux-s390x 0.25.12
- npm @esbuild/linux-x64 0.25.12
- npm @esbuild/netbsd-arm64 0.25.12
- npm @esbuild/netbsd-x64 0.25.12
- npm @esbuild/openbsd-arm64 0.25.12
- npm @esbuild/openbsd-x64 0.25.12
- npm @esbuild/openharmony-arm64 0.25.12
- npm @esbuild/sunos-x64 0.25.12
- npm @esbuild/win32-arm64 0.25.12
- npm @esbuild/win32-ia32 0.25.12
- npm @esbuild/win32-x64 0.25.12
- npm @napi-rs/lzma-linux-x64-gnu 1.5.1
- npm @rollup/rollup-android-arm-eabi 4.63.1
- npm @rollup/rollup-android-arm64 4.63.1
- npm @rollup/rollup-darwin-arm64 4.63.1
- npm @rollup/rollup-darwin-x64 4.63.1
- npm @rollup/rollup-freebsd-arm64 4.63.1
- npm @rollup/rollup-freebsd-x64 4.63.1
- npm @rollup/rollup-linux-arm-gnueabihf 4.63.1
- npm @rollup/rollup-linux-arm-musleabihf 4.63.1
- npm @rollup/rollup-linux-arm64-gnu 4.63.1
- npm @rollup/rollup-linux-arm64-musl 4.63.1
- npm @rollup/rollup-linux-loong64-gnu 4.63.1
- npm @rollup/rollup-linux-loong64-musl 4.63.1
- npm @rollup/rollup-linux-ppc64-gnu 4.63.1
- npm @rollup/rollup-linux-ppc64-musl 4.63.1
- npm @rollup/rollup-linux-riscv64-gnu 4.63.1
- npm @rollup/rollup-linux-riscv64-musl 4.63.1
- npm @rollup/rollup-linux-s390x-gnu 4.63.1
- npm @rollup/rollup-linux-x64-gnu 4.63.1
- npm @rollup/rollup-linux-x64-musl 4.63.1
- npm @rollup/rollup-openbsd-x64 4.63.1
- npm @rollup/rollup-openharmony-arm64 4.63.1
- npm @rollup/rollup-win32-arm64-msvc 4.63.1
- npm @rollup/rollup-win32-ia32-msvc 4.63.1
- npm @rollup/rollup-win32-x64-gnu 4.63.1
- npm @rollup/rollup-win32-x64-msvc 4.63.1
- npm @tauri-apps/cli-darwin-arm64 2.11.4
- npm @tauri-apps/cli-darwin-x64 2.11.4
- npm @tauri-apps/cli-linux-arm-gnueabihf 2.11.4
- npm @tauri-apps/cli-linux-arm64-gnu 2.11.4
- npm @tauri-apps/cli-linux-arm64-musl 2.11.4
- npm @tauri-apps/cli-linux-riscv64-gnu 2.11.4
- npm @tauri-apps/cli-linux-x64-gnu 2.11.4
- npm @tauri-apps/cli-linux-x64-musl 2.11.4
- npm @tauri-apps/cli-win32-arm64-msvc 2.11.4
- npm @tauri-apps/cli-win32-ia32-msvc 2.11.4
- npm @tauri-apps/cli-win32-x64-msvc 2.11.4
- npm is-reference 3.0.3
- npm locate-character 3.0.0
- npm stackback 0.0.2

## App identification icons

`apps/desktop/public/app-icons/0.png` through `19.png` identify, in order: ChatGPT, Claude, Gemini, Microsoft Copilot, Perplexity, Cursor, VS Code, Slack, Microsoft Teams, Gmail, Outlook, Google Docs, Microsoft Word, Notion, Linear, Asana, Jira, Zoom, WhatsApp, and GitHub.

These third-party brand marks remain the property of their respective owners; the project's MIT license does not grant rights to those marks. Their display indicates examples of editable-field destinations, not endorsement, affiliation, or individually certified compatibility. Icons were retrieved on 2026-09-21 from Google's favicon service using the corresponding product domains; the Google Docs icon comes directly from `https://ssl.gstatic.com/docs/documents/images/kix-favicon7.ico`. Assets are bundled locally; the running app does not contact the favicon service.
