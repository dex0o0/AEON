#!/bin/bash

DIR_RELEASE=$(pwd)/target/release
ROOT_BINARY=/usr/bin/AEON
CLI_BINARY=$DIR_RELEASE/aeoncli
APP_NAME="AEON"
BINARY_PATH=$DIR_RELEASE/$APP_NAME
SERVICE_DIR=/etc/systemd/system
GROUP="aeon"
SERVICE_PATH=$SERVICE_DIR/AEON.service
SERVIC_CONF=$(cat <<EOF
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
error_exit(){
  echo "ERROR:$1">&2
  exit 1
}

groupConf(){
  if ! getent group $GROUP &>/dev/null;then
    echo "group:$GROUP ,does not exitst creating it..."
    echo "creating group $GROUP" && sudo groupadd $GROUP || error_exit "Fail to create group"
  fi

  if ! groups $USER | grep -q "\b$GROUP\b"; then
    echo "user:$USER,is not in group $GROUP"
    echo "adding user to group $GROUP" && sudo usermod -a $USER -G $GROUP || error_exit "Failde to add user:$USER to group:$GROUP"
  fi
}
config_cli(){
  if [[ -f $CLI_BINARY ]];then
    echo "moving $CLI_BINARY to $ROOT_BINARY" && sudo mv $CLI_BINARY $ROOT_BINARY || error_exit "can't move $CLI_BINARY to $ROOT_BINARY"
  fi
}
confing_aeon(){
  echo "configing service..."
  echo "$SERVIC_CONF"|sudo tee "$SERVICE_PATH">/dev/null || error_exit "Failed to config $SERVICE_PATH"
  echo "config completed"
}

reload_daemon(){
  echo "reload daemon" && sudo systemctl daemon-reload && sudo systemctl restart "AEON.service"
}

check_conf(){
  data=$(cat $SERVICE_PATH)
  if [[ $data == $SERVIC_CONF ]];then
    echo "$(basename $SERVICE_PATH),already confinged"
    return 0
  else 
    return 1
  fi 
}

if which cargo >/dev/null;then
  echo "building service..."
  if cargo build --release &>/dev/null;then
    echo "build success.."
    if [ -f $BINARY_PATH ];then
      echo "moving file $BINARY_PATH to $ROOT_BINARY" && sudo mv $BINARY_PATH $ROOT_BINARY
    fi
    config_cli 
    echo "creating AEON.service"
    if ! [ -d $SERVICE_DIR ]; then
      echo "make $SERVICE_DIR" && mkdir -p $SERVICE_DIR
    fi
    echo -e "create conf in path \'$SERVICE_DIR\'"
    if ! [ -f $SERVICE_PATH ];then
      echo "create $SERVICE_PATH" && sudo touch $SERVICE_PATH
      echo "create AEON.service"
    fi
    if ! check_conf ; then
      echo "calling fn conf..." && confing_aeon 
    fi
    echo "calling fn group conf..." && groupConf && echo "group confinged" 
    echo "reloading.." && reload_daemon 
    
  else
    echo "<Error building>\n\tPlease check cargo build and next run install file."
  fi
else 
  sys=$(uname -n)
  echo "download cargo and rustup..."
  if [[ $sys == "arch" ]];then
    sudo pacman -S cargo rustup 
  fi
  ./install.AEON.sh
fi
