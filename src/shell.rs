/// Escape a string for safe interpolation inside single-quoted shell contexts.
/// Replaces `'` with `'\''` (end quote, literal quote, reopen quote).
pub fn escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}
