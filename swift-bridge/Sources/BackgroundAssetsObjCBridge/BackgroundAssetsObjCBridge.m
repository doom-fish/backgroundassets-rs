#import "BackgroundAssetsObjCBridge.h"

BAURLDownload *BAXTryMakeURLDownload(
    NSString *identifier,
    NSURLRequest *request,
    BOOL essential,
    NSUInteger fileSize,
    NSString *applicationGroupIdentifier,
    BADownloaderPriority priority,
    NSError * _Nullable * _Nullable error
) {
    @try {
        return [[BAURLDownload alloc] initWithIdentifier:identifier
                                                 request:request
                                               essential:essential
                                                fileSize:fileSize
                              applicationGroupIdentifier:applicationGroupIdentifier
                                                priority:priority];
    } @catch (NSException *exception) {
        if (error != NULL) {
            NSString *message = exception.reason ?: exception.name;
            *error = [NSError errorWithDomain:@"BackgroundAssetsObjCBridge"
                                         code:1
                                     userInfo:@{NSLocalizedDescriptionKey: message}];
        }
        return nil;
    }
}
