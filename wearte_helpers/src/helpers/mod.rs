cfg_if! {
    if #[cfg(wearte_nightly)] {
        #[path = "markup-night.rs"]
        mod markup;
        pub use markup::MarkupAsStr;
    } else {
        mod markup;
        pub use markup::MarkupAsStr;
    }
}
