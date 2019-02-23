cfg_if! {
    if #[cfg(yarte_nightly)] {
        #[path = "markup-night.rs"]
        mod markup;
        pub use markup::MarkupAsStr;
    } else {
        mod markup;
        pub use markup::MarkupAsStr;
    }
}
