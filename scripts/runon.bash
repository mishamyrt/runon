#!/bin/bash
# Due to the limitations of Swift (or rather even Objective C),
# it is not possible to make a fork to create a Daemon process.
# I didn't want to make an XPC service or something similar,
# so made a separate script that manages the Daemon process.
# 
# A search query for those interested in the details: objc_initializeAfterForkError

readonly CONFIG_DIR="$HOME/.config/runon"
readonly CONFIG_FILE="$CONFIG_DIR/config.yaml"
readonly APP_PATH="/usr/local/bin/runon-daemon"
readonly PID_FILE="/tmp/runon.pid"
readonly LOG_FILE="/tmp/runon.log"

config::edit() {
    mkdir -p "$CONFIG_DIR"
    ${EDITOR:=vi} "$CONFIG_FILE"
}

config::path() {
    echo "$CONFIG_FILE"
}

app::run() {
    $APP_PATH
}

app::print() {
    $APP_PATH print
}

daemon::is_running() {
    if [ -f $PID_FILE ]; then
        kill -0 "$(cat $PID_FILE)" &> /dev/null
    else
        false
    fi
}

daemon::start() {
    if daemon::is_running; then
        echo "daemon is already running"
    else
        # shellcheck disable=SC2094
        nohup script -q $LOG_FILE $APP_PATH > $LOG_FILE &
        rm -f "$PID_FILE"
        echo $! > $PID_FILE
        echo "daemon started"
    fi
}

daemon::stop() {
    if daemon::is_running; then
        kill -9 "$(cat $PID_FILE)"
        rm $PID_FILE
        echo "daemon stopped"
    else
        echo "daemon is not yet running "
    fi
}

daemon::restart() {
    daemon::stop
    daemon::start
}

daemon::status() {
    if daemon::is_running; then
        echo "runon is running"
    else
        echo "runon is not running"
    fi
}

daemon::log() {
    if [ ! -f $LOG_FILE ]; then
        echo "Log file is not found"
    else
        tail -f "$LOG_FILE"
    fi
}

display_help() {
   echo "A utility for automating actions on system events."
   echo
   echo "Usage: runon <subcommand>"
   echo
   echo "Subcommands:"
   echo "  run           Runs the application in the current process without daemonization."
   echo "  print         Starts the observer in a special mode that prints all supported events."
   echo "  autostart     Enables, disables, or prints the autostart status."
   echo "  start         Starts the application in the background."
   echo "  stop          Stops the background application."
   echo "  restart       Restarts background application."
   echo "  status        Prints the status of the application."
   echo "  log           Starts the application log viewer in follow mode."
   echo "  config        Opens editing of the configuration file."
   echo "  config-path   Prints the absolute path of the configuration file."
   echo "  help          Prints this message."
   echo
}

case "$1" in
    run)
        app::run
        ;;
    print)
        app::print
        ;;
    autostart)
        if [ -z "$2" ]; then
            $APP_PATH autostart
        else
            $APP_PATH autostart "$2"
        fi
        ;;
    start)
        daemon::start
        ;;
    stop)
        daemon::stop
        ;;
    restart)
        daemon::restart
        ;;
    status)
        daemon::status
        ;;
    log)
        daemon::log
        ;;
    config)
        config::edit
        ;;
    config-path)
        config::path
        ;;
    *)
        display_help
esac