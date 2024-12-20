#include "main.hpp"

// using namespace cv;
using namespace std;

int main(int argc, char* argv [ ]) {
    if (argc > 1) {
        string target = argv[1];
        if (target == "download") {
            fprintf(stderr, "Fetching bulk download link\n");
            CURL* webhandle = curl_easy_init();
            if (!webhandle) {
                cerr << "Could not init curl" << endl;
                return 1;
            }
            string wresponse = "";
            curl_easy_setopt(webhandle, CURLOPT_URL, "https://api.scryfall.com/bulk-data");
            curl_easy_setopt(webhandle, CURLOPT_WRITEFUNCTION, [&](char* data, size_t size, size_t nmemb, void* udata) {
                cout << data << endl;
                size_t realsize = size * nmemb;
                string* out = (string*)udata;
                out->append(data);
                return realsize;
            });
            curl_easy_setopt(webhandle, CURLOPT_WRITEDATA, (void*)&wresponse);
            CURLcode resp = curl_easy_perform(webhandle);
            if (resp != CURLE_OK) {
                cerr << "cURL error" << endl;
                return 1;
            }
            curl_easy_cleanup(webhandle);
            fprintf(stderr, "\n\n%s", wresponse.c_str());
        }
        else if (target == "hash") {
            fprintf(stderr, "pregen hash target\n");
        }
    }
}