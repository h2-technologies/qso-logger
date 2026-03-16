mod config;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_write_and_read() {
        let config = config::Config::new();

        let path = "test_config.json";
        config.save_to_file(path).expect("Failed to save config");

        let loaded_config = config::Config::load_from_file(path).expect("Failed to load config");
        assert_eq!(config.callsign, loaded_config.callsign);
        assert_eq!(config.maidenhead, loaded_config.maidenhead);
        assert_eq!(config.qrz_upload, loaded_config.qrz_upload);
        assert_eq!(config.lotw_upload, loaded_config.lotw_upload);
        assert_eq!(config.eqsl_upload, loaded_config.eqsl_upload);

        std::fs::remove_file(path).expect("Failed to clean up test file");
    }
}
