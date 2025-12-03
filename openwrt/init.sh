#!/bin/sh /etc/rc.common

USE_PROCD=1
START=95
STOP=01

start_service() {
  procd_open_instance
  procd_set_param env RUST_LOG="info"
  procd_set_param command /usr/bin/forward_bot
  procd_set_param user nobody
  procd_set_param respawn 3600 5 5
  procd_set_param stdout 1
  procd_set_param stderr 1
  procd_close_instance
}
