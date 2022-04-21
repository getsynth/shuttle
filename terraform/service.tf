resource "aws_network_interface" "backend" {
  subnet_id = aws_subnet.backend_b.id
}

resource "aws_eip" "backend" {
  vpc               = true
  network_interface = aws_network_interface.backend.id
}

resource "aws_network_interface_sg_attachment" "backend" {
  security_group_id    = aws_security_group.unreasonable.id
  network_interface_id = aws_network_interface.backend.id
}

resource "aws_iam_instance_profile" "backend" {
  name = "backend-profile"
  role = "BackendAPIRole"
}

resource "aws_lb_target_group_attachment" "api" {
  target_group_arn = aws_lb_target_group.api.arn
  target_id        = aws_instance.backend.id
  port             = var.api_container_port
}

resource "aws_lb_target_group_attachment" "user" {
  target_group_arn = aws_lb_target_group.user.arn
  target_id        = aws_instance.backend.id
  port             = var.proxy_container_port
}

resource "aws_lb_target_group_attachment" "postgres" {
  target_group_arn = aws_lb_target_group.postgres.arn
  target_id        = aws_instance.backend.id
  port             = var.postgres_container_port
}

resource "aws_instance" "backend" {
  ami           = "ami-072db068702487a87" # unveil-backend-ami-20220313
  instance_type = "c6g.4xlarge"

  monitoring = true

  availability_zone = "eu-west-2b"

  iam_instance_profile = aws_iam_instance_profile.backend.id

  network_interface {
    network_interface_id = aws_network_interface.backend.id
    device_index         = 0
  }

  user_data                   = data.cloudinit_config.backend.rendered
  user_data_replace_on_change = false
}

locals {
  opt_shuttle_content = templatefile(
    "${path.module}/systemd/system/opt-shuttle.mount.tftpl",
    {
      dns_name = aws_efs_file_system.user_data.dns_name,
      data_dir = local.data_dir
    }
  )
  shuttle_backend_content = templatefile(
    "${path.module}/systemd/system/shuttle-backend.service.tftpl",
    {
      data_dir             = local.data_dir,
      docker_image         = local.docker_image,
      pg_password          = var.postgres_password,
      shuttle_admin_secret = var.shuttle_admin_secret,
      proxy_fqdn           = var.proxy_fqdn,
      shuttle_initial_key  = random_string.initial_key.result
    }
  )
}

data "cloudinit_config" "backend" {
  gzip          = false
  base64_encode = false

  part {
    content_type = "text/cloud-config"
    content = templatefile(
      "${path.module}/misc/cloud-config.yaml",
      {
        opt_shuttle_content     = base64encode(local.opt_shuttle_content),
        shuttle_backend_content = base64encode(local.shuttle_backend_content)
      }
    )
    filename = "cloud-config.yaml"
  }
}
