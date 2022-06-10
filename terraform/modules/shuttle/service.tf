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
  role = aws_iam_role.backend.name
}

resource "aws_iam_role" "backend" {
  name        = "BackendAPIRole"
  path        = "/"
  description = "Allows EC2 instances to call AWS services on your behalf."

  assume_role_policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Action": "sts:AssumeRole",
            "Principal": {
               "Service": "ec2.amazonaws.com"
            },
            "Effect": "Allow",
            "Sid": ""
        }
    ]
}
EOF
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

data "aws_ami" "ubuntu" {
  most_recent = true

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-focal-20.04-arm64-server-20220511"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }

  owners = ["099720109477"] # Canonical
}

resource "aws_instance" "backend" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_type

  monitoring = true

  availability_zone = "eu-west-2b"

  iam_instance_profile = aws_iam_instance_profile.backend.id

  network_interface {
    network_interface_id = aws_network_interface.backend.id
    device_index         = 0
  }

  root_block_device {
    delete_on_termination = true
    encrypted             = false
    volume_size           = 64
    volume_type           = "gp2"
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
      docker_image         = local.docker_backend_image,
      shuttle_admin_secret = var.shuttle_admin_secret,
      proxy_fqdn           = var.proxy_fqdn,
      shuttle_initial_key  = random_string.initial_key.result
    }
  )
  shuttle_provisioner_content = templatefile(
    "${path.module}/systemd/system/shuttle-provisioner.service.tftpl",
    {
      data_dir     = local.data_dir,
      docker_image = local.docker_provisioner_image,
      pg_password  = var.postgres_password,
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
        opt_shuttle_content         = base64encode(local.opt_shuttle_content),
        shuttle_backend_content     = base64encode(local.shuttle_backend_content)
        shuttle_provisioner_content = base64encode(local.shuttle_provisioner_content)
      }
    )
    filename = "cloud-config.yaml"
  }
}
