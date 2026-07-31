POST {{base_url}}/user/login
[Options]
variable: base_url=http://localhost:3000
variable: username=test_user
variable: password=111111
output: -

{
    "username":"{{username}}",
    "password":"{{password}}"
}
