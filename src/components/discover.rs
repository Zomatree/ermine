use freya::prelude::*;

#[derive(PartialEq)]
pub struct Discover {}

impl Component for Discover {
    fn render(&self) -> impl IntoElement {
        rect()
            .background(0xff292a2f)
            .expanded()
            .padding(16.)
            .child("Not Implemented")
//             .child(
//                 WebView::new("https://stt.gg?embedded=true")
//                     .expanded()
//                     .on_created(|builder| {
//                         builder.with_initialization_script(
//                             r##"
// let observer = new MutationObserver((_, o) => {
//     if (document.body != null) {
//         document.body.style.background = "#292a2f";
//     };

//     const container = document.querySelector("#__next > div > div.Page__PageContainer-sc-xp2hl6-1.bqrbnZ");
//     const base = document.querySelector("#__next > div");
//     const sidebar = document.querySelector("#__next > div > div.Sidebar__Base-sc-1uh6eub-0.cRCSBm");

//     if (container == null || base == null || sidebar == null) {
//         return
//     }

//     base.style.background = "#292a2f";
//     sidebar.style.borderRadius = "16px 0px 0px 16px";
//     sidebar.style.borderRadius = "16px 0px 0px 16px";
//     container.style.background = "var(--background)";

//     o.disconnect();
// });

// observer.observe(document, { attributes: true, childList: true, subtree: true });
// "##,
//                         ).with_background_color((41, 42, 47, 255))
//                     }),
//             )
    }
}
