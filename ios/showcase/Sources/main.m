// iOS entry for the plev showcase demo.
//
// winit owns the iOS application lifecycle: showcase_ios_main (exported by the
// Rust staticlib) builds the winit event loop and calls UIApplicationMain,
// which never returns. This main is just the C entry the linker expects.
#import <UIKit/UIKit.h>

extern void showcase_ios_main(void);

int main(int argc, char *argv[]) {
    @autoreleasepool {
        showcase_ios_main();
    }
    return 0;
}
