// iOS entry for the plev uigallery app.
//
// winit owns the iOS application lifecycle: uigallery_ios_main (exported by the
// Rust staticlib) builds the winit event loop and calls UIApplicationMain,
// which never returns. This main is just the C entry the linker expects.
#import <UIKit/UIKit.h>

extern void uigallery_ios_main(void);

int main(int argc, char *argv[]) {
    @autoreleasepool {
        uigallery_ios_main();
    }
    return 0;
}
