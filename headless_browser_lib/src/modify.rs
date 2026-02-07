use hyper::body::Bytes;

/// modify the json output for the bytes hosting. The headless instance cannot accept external request so we use the proxy.
pub(crate) fn modify_json_output(body_bytes: Bytes) -> Bytes {
    modify_json_output_with_host(body_bytes, crate::HOST_NAME.as_bytes())
}

fn modify_json_output_with_host(body_bytes: Bytes, replacement_host: &[u8]) -> Bytes {
    let buffer = body_bytes.as_ref();

    let (target_port, replacement_port) = *crate::TARGET_REPLACEMENT;

    let mut modified_buffer = buffer.to_vec();

    // Replace occurrences of localhost and 127.0.0.1
    for target_host in [b"127.0.0.1".as_slice(), b"localhost".as_slice()] {
        if target_host == replacement_host {
            continue;
        }

        let mut next_buffer = Vec::with_capacity(
            modified_buffer.len() + (replacement_host.len() - target_host.len()),
        );
        let mut start = 0;

        while let Some(pos) = modified_buffer[start..]
            .windows(target_host.len())
            .position(|window| window == target_host)
        {
            next_buffer.extend_from_slice(&modified_buffer[start..start + pos]);
            next_buffer.extend_from_slice(replacement_host);
            start += pos + target_host.len();
        }

        next_buffer.extend_from_slice(&modified_buffer[start..]);
        modified_buffer = next_buffer;
    }

    // Now handle the port replacement
    let mut final_buffer =
        Vec::with_capacity(modified_buffer.len() + (replacement_port.len() - target_port.len()));
    let mut start = 0;

    while let Some(pos) = modified_buffer[start..]
        .windows(target_port.len())
        .position(|window| window == target_port)
    {
        final_buffer.extend_from_slice(&modified_buffer[start..start + pos]);
        final_buffer.extend_from_slice(replacement_port);
        start += pos + target_port.len();
    }
    final_buffer.extend_from_slice(&modified_buffer[start..]);

    final_buffer.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_localhost_and_loopback_in_json() {
        let (target_port, replacement_port) = *crate::TARGET_REPLACEMENT;
        let replacement_host = b"headless-browser";

        let input = format!(
            "{{\"ws1\":\"ws://127.0.0.1{}/devtools/browser/a\",\"ws2\":\"ws://localhost{}/devtools/browser/b\"}}",
            std::str::from_utf8(target_port).unwrap(),
            std::str::from_utf8(target_port).unwrap()
        );

        let output = modify_json_output_with_host(Bytes::from(input), replacement_host);
        let output_str = std::str::from_utf8(&output).unwrap();

        assert!(!output_str.contains("127.0.0.1"));
        assert!(!output_str.contains("localhost"));
        assert!(output_str.contains("headless-browser"));
        assert!(!output_str.contains(std::str::from_utf8(target_port).unwrap()));
        assert!(output_str.contains(std::str::from_utf8(replacement_port).unwrap()));
    }
}
