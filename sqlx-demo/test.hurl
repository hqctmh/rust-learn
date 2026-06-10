# 创建用户成功。
POST {{base_url}}/v1/user
[Options]
variable: base_url=http://localhost:8080
variable: username=demo_user_001
variable: password=passw0rd
{
  "username": "{{username}}",
  "password": "{{password}}"
}
HTTP 204

# 重复用户名返回冲突。
POST {{base_url}}/v1/user
{
  "username": "{{username}}",
  "password": "{{password}}"
}
HTTP 409
[Asserts]
jsonpath "$.message" == "username taken"

# 非法用户名返回请求体校验错误。
POST {{base_url}}/v1/user
{
  "username": "ab",
  "password": "{{passwordew"
}
HTTP 422
[Asserts]
jsonpath "$.message" == "validation error in request body"
