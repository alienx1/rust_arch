use tailwind_fuse::merge::tw_merge_slice;

/// Utility function to merge Tailwind CSS classes, resolving conflicts (like shadcn `cn(...)`).
pub fn cn(classes: &[&str]) -> String {
    tw_merge_slice(classes)
}
