data "aws_route53_zone" "user" {
  name = "shuttleapp.rs."
}

resource "aws_acm_certificate" "user" {
  domain_name = "shuttleapp.rs"

  subject_alternative_names = [
    "*.shuttleapp.rs"
  ]

  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_route53_record" "user" {
  for_each = {
    for dvo in aws_acm_certificate.user.domain_validation_options : dvo.domain_name => {
      name    = dvo.resource_record_name
      record  = dvo.resource_record_value
      type    = dvo.resource_record_type
      zone_id = data.aws_route53_zone.user.zone_id
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = each.value.zone_id
}

resource "aws_acm_certificate_validation" "user" {
  certificate_arn = aws_acm_certificate.user.arn
  validation_record_fqdns = [for record in aws_route53_record.user : record.fqdn]
}

data "aws_route53_zone" "root" {
  name = "shuttle.rs."
}

resource "aws_acm_certificate" "api" {
  domain_name = var.api_fqdn

  validation_method = "DNS"

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_route53_record" "api" {
  for_each = {
    for dvo in aws_acm_certificate.api.domain_validation_options : dvo.domain_name => {
      name    = dvo.resource_record_name
      record  = dvo.resource_record_value
      type    = dvo.resource_record_type
      zone_id = data.aws_route53_zone.root.zone_id
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = each.value.zone_id
}

resource "aws_acm_certificate_validation" "api" {
  certificate_arn = aws_acm_certificate.api.arn
  validation_record_fqdns = [for record in aws_route53_record.api : record.fqdn]
}
