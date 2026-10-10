#include "libmain.h"

#include <curl/curl.h>

#include <stdint.h>
#include <stdio.h>
#include <string.h>

// Two modes, so one fixture serves both CI rows.
//
// With no argument the host only proves the graph links and initializes, which is all the
// device row can do: ios-arm64 needs signing, provisioning and hardware to run anywhere.
//
// With `--live` it performs one real HTTPS GET through the linked libcurl. Compile/link
// evidence proves the archive is the SecTrust build and that Security.framework resolves; it
// cannot prove a request VERIFIES, because default iOS HTTPS leaves CURLOPT_CAINFO unset so
// SecTrust supplies the anchors. A regression that compiles, links, and still fails TLS is
// invisible without a transfer (issue #873).
//
// The transfer is driven from C rather than from the PHP half deliberately. An iOS build is a
// library: `elephc_init()` runs no top-level code, and an exported function that reached curl
// is refused by the export gate, which requires every export to be recoverable. So the PHP
// side settles which archives are linked, and this side exercises them. What that leaves
// unchecked on iOS is elephc's own curl surface at run time; the Linux and macOS codegen
// shards cover that.
static size_t discard(char *data, size_t size, size_t count, void *user) {
    (void)data;
    (void)user;
    return size * count;
}

static int live_https_get(void) {
    CURL *easy = curl_easy_init();
    if (easy == NULL) {
        fprintf(stderr, "curl_easy_init failed\n");
        return 3;
    }

    // No CAINFO/CAPATH on purpose: that is what makes libcurl hand the chain to SecTrust.
    curl_easy_setopt(easy, CURLOPT_URL, "https://example.com/");
    curl_easy_setopt(easy, CURLOPT_SSL_VERIFYPEER, 1L);
    curl_easy_setopt(easy, CURLOPT_SSL_VERIFYHOST, 2L);
    curl_easy_setopt(easy, CURLOPT_TIMEOUT, 30L);
    curl_easy_setopt(easy, CURLOPT_WRITEFUNCTION, discard);

    CURLcode rc = curl_easy_perform(easy);
    if (rc != CURLE_OK) {
        // CURLE_PEER_FAILED_VERIFICATION (60) is what a SecTrust regression looks like.
        fprintf(stderr, "live HTTPS GET failed: %d (%s)\n", (int)rc, curl_easy_strerror(rc));
        curl_easy_cleanup(easy);
        return 3;
    }

    long status = 0;
    curl_easy_getinfo(easy, CURLINFO_RESPONSE_CODE, &status);
    curl_easy_cleanup(easy);

    if (status != 200) {
        fprintf(stderr, "live HTTPS GET verified but answered %ld\n", status);
        return 3;
    }
    printf("live HTTPS GET verified through SecTrust: %ld\n", status);
    return 0;
}

int main(int argc, char **argv) {
    if (elephc_abi_version() != ELEPHC_ABI_VERSION) return 1;
    if (elephc_init() != ELEPHC_STATUS_OK) return 2;

    (void)ios_curl_link_smoke();

    int status = 0;
    if (argc > 1 && strcmp(argv[1], "--live") == 0) {
        status = live_https_get();
    }

    elephc_shutdown();
    return status;
}
