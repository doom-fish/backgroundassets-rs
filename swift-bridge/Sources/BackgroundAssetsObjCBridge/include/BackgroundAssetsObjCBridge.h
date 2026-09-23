#import <Foundation/Foundation.h>
#import <BackgroundAssets/BackgroundAssets.h>

NS_ASSUME_NONNULL_BEGIN

BAURLDownload * _Nullable BAXTryMakeURLDownload(
    NSString *identifier,
    NSURLRequest *request,
    BOOL essential,
    NSUInteger fileSize,
    NSString *applicationGroupIdentifier,
    BADownloaderPriority priority,
    NSError * _Nullable * _Nullable error
) API_AVAILABLE(macos(13.3));

NS_ASSUME_NONNULL_END
