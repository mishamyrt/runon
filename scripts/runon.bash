#!/bin/bash
# objc_initializeAfterForkError

CONFIG_DIR="$HOME/.config/runon"
CONFIG_FILE="$CONFIG_DIR/config.yaml"
APP_PATH="/usr/local/bin/runon-daemon"
PID_FILE="/tmp/runon.pid"

editConfig() {
    mkdir -p "$CONFIG_DIR"
    ${EDITOR:=vi} "$CONFIG_FILE"
}

isRunning() {
    if [ -f $PID_FILE ]; then
        kill -0 "$(cat $PID_FILE)" &> /dev/null
    else
        false
    fi
}

status() {
    if isRunning; then
        echo "runon is running"
    else
        echo "runon is not running"
    fi
}

start() {
    if isRunning; then
        echo "runon daemon is already running"
    else
        nohup $APP_PATH &> /dev/null &
        rm -f "$PID_FILE"
        echo $! > $PID_FILE
        echo "runon daemon started"
    fi
}

stop() {
    if isRunning; then
        kill "$(cat $PID_FILE)"
        rm $PID_FILE
        echo "runon daemon stopped"
    else
        echo "runon daemon is not running"
    fi
}

restart() {
    stop
    start
}

case "$1" in
    run)
        $APP_PATH run
        ;;
    autostart)
        if [ -z "$2" ]; then
            $APP_PATH autostart
        else
            $APP_PATH autostart "$2"
        fi
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    print)
        $APP_PATH print
        ;;
    config)
        editConfig
        ;;
    config-path)
        echo "$CONFIG_FILE"
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: runon run|start|stop|restart|autostart|print"
esac