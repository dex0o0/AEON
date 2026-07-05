#!/bin/bash

set -emu

RED='\033[31m'
BLUE='\033[34m'
GREEN='\033[32m'
YELLOW='\033[33m'
BOLD='\033[1m'
NC='\033[0m'

LOGROTATE_PATH=/etc/logrotate.d/aeon
ROOT_DIR=/usr/bin/
GUARD_BIN=/usr/bin/aeon_guardian
DIR_RELEASE=$(pwd)/target/release
ROOT_BINARY=/usr/bin/AEON
CLI_BINARY=$DIR_RELEASE/aeoncli
GUARD_NAME="aeon_guardian"
APP_NAME="AEON"
BINARY_PATH=$DIR_RELEASE/$APP_NAME
GUARD_BIN_PATH=$DIR_RELEASE/$GUARD_NAME
SERVICE_DIR=/etc/systemd/system
GROUP="aeon"
SERVICE_PATH=$SERVICE_DIR/AEON.service
GUARD_SERVICE_PATH=$SERVICE_DIR/aeon_guardian.service

AEON_SERVICE_CONF=$(
  cat <<EOF
[Unit]
Description=$APP_NAME daemon service
After=network.target

[Service]
ExecStart=$ROOT_BINARY
Restart=always
RestartSec=3

User=$USER
Group=$GROUP

StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
)

GUARD_SERVICE_CONF=$(
  cat <<EOF
[Unit]
Description=AEON Guardian Service
After=network.target AEON.service

[Service]
ExecStart=$GUARD_BIN
Restart=always
RestartSec=3

User=$USER
Group=$GROUP

StandardOutput=journal
StandardError=journal

[install]
WantedBy=multi-user.target
EOF
)
REAL_HOME=$(eval echo "~${SUDO_USERS:-$USER}")

LOG_R_CONF=$(
  cat <<EOF
$REAL_HOME/.log/dex_daemon/*.log{
daily
rotate 7
compress
delaycompress
missingok
notifempty
maxsize 1M
create 644 $USER $USER
}
EOF
)

error_exit() {
  eprint $1 >&2
  exit 1
}

printed() {
  echo -e "${GREEN}=> ${NC}${BOLD}$1${NC}"
}
yprint() {
  echo -e "[${GREEN}OK${NC}]:${BOLD}${BLUE}$1${NC}"
}
eprint() {
  echo -e "[${RED}ERROR${NC}]:${BOLD}${YELLOW}$1${NC}"
}

#config logrotate
config_logrotate() {
  printed "configing logrotate..."
  if ! [ -f $LOGROTATE_PATH ]; then
    printed "creating" && sudo touch $LOGROTATE_PATH || error_exit "Error to create logrotate file \{$LOGROTATE_PATH\}"
    printed "config logrotate.." && echo "$LOG_R_CONF" | sudo tee "$LOGROTATE_PATH" >/dev/null || error_exit "Error config logrotate"
    yprint "---config completed---"
  else
    printed "config loger"
    da=$(cat $LOGROTATE_PATH)
    if ! [[ $da == $LOG_R_CONF ]]; then
      sudo rm $LOGROTATE_PATH && config_logrotate || error_exit "Error to conf $LOGROTATE_PATH"
    fi
  fi
}

#config groups and permisions
groupConf() {
  if ! getent group $GROUP &>/dev/null; then
    printed "group:$GROUP ,does not exitst creating it..."
    printed "creating group $GROUP" && sudo groupadd $GROUP || error_exit "Fail to create group"
  fi

  if ! groups $USER | grep -q "\b$GROUP\b"; then
    printed "user:$USER,is not in group $GROUP"
    printed "adding user to group $GROUP" && sudo usermod -a $USER -G $GROUP || error_exit "Failde to add user:$USER to group:$GROUP"
  fi
}

#config aeoncli
config_cli() {
  if [[ -f $CLI_BINARY ]]; then
    printed "moving $CLI_BINARY to $ROOT_DIR" && sudo mv $CLI_BINARY $ROOT_DIR || error_exit "can't move $CLI_BINARY to $ROOT_DIR"
  fi
}

#config AEON service
confing_aeon() {
  printed "configing service..."
  echo "$AEON_SERVICE_CONF" | sudo tee "$SERVICE_PATH" >/dev/null || error_exit "Failed to config $SERVICE_PATH"
  yprint "config completed"
}

#config Guardian
config_guardian() {
  printed "moving $GUARD_BIN_PATH" to $GUARD_BIN && sudo mv $GUARD_BIN_PATH $GUARD_BIN || error_exit "can't move $GUARD_BIN_PATH to $GUARD_BIN"
  printed "config Guardian"
  echo "$GUARD_SERVICE_CONF" | sudo tee "$GUARD_SERVICE_PATH" >/dev/null || error_exit "Failed to config $GUARD_SERVICE_PATH"
  printed "config completed"
}

#reload AEON service
reload_daemon() {
  printed "reload daemon" && sudo systemctl daemon-reload && sudo systemctl restart "AEON.service" && sudo systemctl restart "aeon_guardian.service"
}

#check function
check_conf() {
  data=$(cat $SERVICE_PATH)
  if [[ $data == $AEON_SERVICE_CONF ]]; then
    yprint "$(basename $SERVICE_PATH),already confinged"
    return 0
  else
    return 1
  fi
}

if which cargo >/dev/null; then
  printed "building service..."
  if cargo build --release; then
    yprint "build success.."
    if [ -f $BINARY_PATH ]; then
      printed "moving file $BINARY_PATH to $ROOT_BINARY" && sudo mv $BINARY_PATH $ROOT_BINARY
    fi
    config_cli
    printed "creating AEON.service"
    if ! [ -d $SERVICE_DIR ]; then
      printed "make $SERVICE_DIR" && mkdir -p $SERVICE_DIR
    fi
    printed "create conf in path \'$SERVICE_DIR\'"
    if ! [ -f $SERVICE_PATH ]; then
      eprint "create $SERVICE_PATH" && sudo touch $SERVICE_PATH
      yprint "create AEON.service"
    fi
    config_guardian
    if ! check_conf; then
      printed "calling fn conf..." && confing_aeon
    fi
    printed "calling fn group conf..." && groupConf && yprint "group confinged"
    printed "calling fn loger config.." && config_logrotate && yprint "loger configed"
    printed "reloading.." && reload_daemon
    yprint "<---install successfully--->"
  else
    eprint "<Error building>\n\tPlease check cargo build and next run install file."
  fi
else
  sys=$(uname -n)
  printed "download cargo and rustup..."
  if [[ $sys == "arch" ]]; then
    sudo pacman -S cargo rustup
  fi
  ./install.AEON.sh
fi
