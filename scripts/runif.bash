#!/bin/bash
# objc_initializeAfterForkError

APP_PATH="/usr/local/bin/runif-daemon"
PID_FILE="/tmp/runif.pid"

isRunning() {
    if [ -f $PID_FILE ]; then
        kill -0 "$(cat $PID_FILE)" &> /dev/null
    else
        false
    fi
}

start() {
    if isRunning; then
        echo "runif daemon is already running"
    else
        nohup $APP_PATH &> /dev/null &
        rm -f "$PID_FILE"
        echo $! > $PID_FILE
        echo "runif daemon started"
    fi
}

stop() {
    if isRunning; then
        kill "$(cat $PID_FILE)"
        rm $PID_FILE
        echo "runif daemon stopped"
    else
        echo "runif daemon is not running"
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
    *)
        echo "Usage: runif run|start|stop|restart|autostart"
esac