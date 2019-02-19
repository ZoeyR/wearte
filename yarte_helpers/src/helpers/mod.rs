cfg_if! {
    if #[cfg(yarte_nightly)] {
        #[path = "markup-night.rs"]
        mod markup;
        pub use markup::MarkupDisplay;
    } else {
        mod markup;
        pub use markup::MarkupDisplay;
    }
}
