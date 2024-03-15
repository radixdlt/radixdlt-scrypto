pub fn list_item_prefix(last: bool) -> &'static str {
    if last {
        "└─"
    } else {
        "├─"
    }
}
