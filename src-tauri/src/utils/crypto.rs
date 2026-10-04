/// 计算字符串的 MD5
pub fn md5_hex(data: &str) -> String {
    format!("{:x}", md5::compute(data.as_bytes()))
}

/// WBI 签名参数混合密钥（B站混肴算法）
const MIXIN_KEY_ENC_TAB: [u8; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49,
    33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40,
    61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11,
    36, 20, 52, 44, 34,
];

/// 从原始密钥生成混肴密钥（取前 32 字符根据映射表重排）
pub fn get_mixin_key(raw_key: &str) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .filter_map(|&idx| raw_key.chars().nth(idx as usize))
        .collect()
}

/// 对请求参数做 WBI 签名
pub fn sign_wbi(params: &mut Vec<(String, String)>, mixin_key: &str) {
    params.sort_by(|a, b| a.0.cmp(&b.0));
    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let sign_str = format!("{}{}", query, mixin_key);
    let wbi_sign = md5_hex(&sign_str);

    params.push(("w_rid".to_string(), wbi_sign));
    params.push(("wts".to_string(), current_timestamp().to_string()));
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_hex() {
        assert_eq!(md5_hex("hello"), "5d41402abc4b2a76b9719d911017c592");
    }

    #[test]
    fn test_get_mixin_key() {
        let raw = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@";
        let key = get_mixin_key(raw);
        assert_eq!(key.len(), 32);
    }
}