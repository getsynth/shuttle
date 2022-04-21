output "container_repository_url" {
  value = aws_ecr_repository.backend.repository_url
}

output "api_url" {
  value       = aws_apigatewayv2_domain_name.backend.id
  description = "URL to connect to the api"
}

output "user_content_host" {
  value = aws_lb.user.dns_name
}

output "initial_user_key" {
  value       = random_string.initial_key.result
  description = "Key given to the initial shuttle user"
}
