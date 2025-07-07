pub trait Format {
    /// Format bytes to KB/MB/GB presentation
    fn bytes(self) -> String;
}

impl Format for u64 {
    fn bytes(self) -> String {
        const KB: f32 = 1024.0;
        const MB: f32 = KB * KB;
        const GB: f32 = MB * KB;

        let f = self as f32;

        if f < KB {
            format!("{self} B")
        } else if f < MB {
            format!("{:.2} KB", f / KB)
        } else if f < GB {
            format!("{:.2} MB", f / MB)
        } else {
            format!("{:.2} GB", f / GB)
        }
    }
}
