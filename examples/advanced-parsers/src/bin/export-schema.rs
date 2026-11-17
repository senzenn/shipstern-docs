// #region export-schema
use shipstern_proc_macro::include_shipstern_parser;

include_shipstern_parser!("padded-envelope.json");

fn main() -> std::io::Result<()> {
    std::fs::write("padded_envelope.proto", padded_envelope::PROTOBUF_SCHEMA)
}
// #endregion export-schema
