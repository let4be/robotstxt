// Copyright 2020 Folyd
// Copyright 1999 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#ifndef RUST_ROBOTSTXT_H
#define RUST_ROBOTSTXT_H

#ifdef __cplusplus
extern "C"{
#endif

bool is_user_agent_allowed(const char *robotstxt, const char *useragent, const char *url);

bool is_user_agent_allowed_caching(const char *robotstxt, const char *useragent, const char *url);

bool is_valid_user_agent_to_obey(const char *useragent);

#ifdef __cplusplus
}
#endif

#endif //RUST_ROBOTSTXT_H
