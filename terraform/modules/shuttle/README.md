# AWS shuttle module
This module contains all the resources needed to deploy shuttle on AWS. The basic architecture is to create:
1. A single EC2 instance to run shuttle and PostgresDB
1. Two Route53 zones - one for the shuttle api; another to reach user services hosted by shuttle (called the proxy)
1. Three Load Balancers - one for the api, proxy, and PostgresDB respectively

## Usage guide
The following terraform can be used as a starting point for using this module:

```tf
module "shuttle" {
  source = "github.com/shuttle-hq/shuttle/terraform/modules/shuttle"

  api_fqdn             = "api.test.shuttle.rs"
  proxy_fqdn           = "test.shuttleapp.rs"
  postgres_password    = "password"
  shuttle_admin_secret = "12345"
}

output "api_name_servers" {
  value = module.shuttle.api_name_servers
}

output "user_name_servers" {
  value = module.shuttle.user_name_servers
}

output "initial_user_key" {
  value       = module.shuttle.initial_user_key
  description = "Key given to the initial shuttle user"
}
```

The shuttle api will be reachable at `api_fqdn` while hosted services will be subdomains of `proxy_fqdn`. The `postgres_password` sets the root password for Postgres and `shuttle_admin_secret` will be the secret needed to add more user keys to shuttle by an admin user. Shuttle does create the first user key though. This key is stored in the `initial_user_key` output variable.

Just running `terraform apply` for the first time will fail since SSl certificates will be created for the api and proxy domains which will be verified. This verification will fail since it uses DNS that will be missing on first setup. So for first setups rather run the following:

``` sh
terraform apply --target module.shuttle.aws_route53_zone.user --target module.shuttle.aws_route53_zone.api
```

This command will create just the DNS zones needed for the api and proxy. Now use the `api_name_servers` and `user_name_servers` outputs from this module to manually add NS records for `api_fqdn` and `proxy_fqdn` in your DNS provider respectively.

Once these records have propagated, a `terraform apply` command will succeed.

