# Devices
[![][img_version]][crates] [![][img_doc]][doc] [![][img_license]][license] [![][img_downloads]][crates]

`soupy` is a library for parsing and querying like BeautifulSoup.

## Cargo Features

- `html`: Support for HTML. Enabled by default.
  - `html-lenient`: Error-tolerant HTML parser. Slow. Enabled by default.
  - `html-strict`: Simple, fast HTML parser. Enabled by default.
- `xml`: Support for XML. Enabled by default.
- `regex`: Support for regex matching in queries. Enabled by default.

## License

`soupy` is dual-licensed under MIT and Apache-2.0.

[img_version]: https://img.shields.io/crates/v/soupy.svg
[img_doc]: https://img.shields.io/badge/rust-documentation-blue.svg
[img_license]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[img_downloads]:https://img.shields.io/crates/d/soupy.svg

[crates]: https://crates.io/crates/soupy
[doc]: https://docs.rs/soupy
[license]: https://github.com/hankjordan/soupy#license
