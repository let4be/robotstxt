#include "rust_robotstxt.h"
#include <string>
#include <iostream>

using namespace std;

int main() {
    const string robotstxt =
            "user-agent: FooBot\n"
            "disallow: /\n";
    bool result = is_user_agent_allowed(robotstxt.c_str(), "FooBot", "");
    cout << result << endl;

    bool result2 = is_user_agent_allowed_caching(robotstxt.c_str(), "FooBot", "");
    cout << result2 << endl;

    cout << is_valid_user_agent_to_obey("Foobot") << endl;
}
