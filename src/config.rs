use crate::error::Result;
use std::path::Path;

pub fn generate_sedna_config(data_dir: &Path, port: u16) -> Result<String> {
    // Use port number to derive a unique os_primitives_id_min_bound
    // This ensures multiple instances don't conflict on shared memory/semaphores
    // Add a large offset to avoid conflicts with other system resources
    let os_primitives_id = 10000 + (port as i32 * 100);

    let config = format!(
        r#"<?xml version="1.0" standalone="yes"?>
<sednaconf>
  <sedna_data>{}</sedna_data>
  <os_primitives_id_min_bound>{}</os_primitives_id_min_bound>
  <listen_address>localhost</listen_address>
  <listener_port>{}</listener_port>
  <ping_port>{}</ping_port>
  <event_log_level>1</event_log_level>
  <keep_alive_timeout>0</keep_alive_timeout>
  <session_stack_depth>4000</session_stack_depth>
</sednaconf>
"#,
        data_dir.display(),
        os_primitives_id,
        port,
        port + 101 // ping_port is typically listener_port + 101
    );

    Ok(config)
}
