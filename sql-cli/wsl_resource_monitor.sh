#!/bin/bash

# WSL Resource Monitor Script
# Usage: ./wsl_resource_monitor.sh [interval_seconds]
# Default interval: 5 seconds

INTERVAL=${1:-5}
LOGFILE="/tmp/wsl_resources.log"

print_header() {
    clear
    echo "================================================================"
    echo "               WSL RESOURCE MONITOR"
    echo "================================================================"
    echo "Poll interval: ${INTERVAL}s | Log: $LOGFILE | Press Ctrl+C to exit"
    echo "================================================================"
    echo
}

get_memory_info() {
    echo "🧠 MEMORY STATUS"
    echo "----------------"
    free -h | grep -E "(Mem|Swap):"
    
    # Memory percentage
    local mem_used=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
    local swap_used=$(free | grep Swap | awk '{print int($3/$2 * 100)}')
    echo "Memory Usage: ${mem_used}% | Swap Usage: ${swap_used}%"
    echo
}

get_cpu_info() {
    echo "⚡ CPU STATUS"
    echo "-------------"
    local load=$(cat /proc/loadavg | awk '{print $1, $2, $3}')
    local cpu_count=$(nproc)
    echo "Load Average: $load (${cpu_count} cores)"
    
    # CPU usage from top
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    echo "CPU Usage: ${cpu_usage}%"
    echo
}

get_disk_info() {
    echo "💾 DISK STATUS"
    echo "--------------"
    echo "Main partitions:"
    df -h | grep -E "(/dev/sd[d-i]|/mnt/[cd])" | while read line; do
        local usage=$(echo "$line" | awk '{print $5}' | tr -d '%')
        local mount=$(echo "$line" | awk '{print $6}')
        local size=$(echo "$line" | awk '{print $2}')
        local used=$(echo "$line" | awk '{print $3}')
        local avail=$(echo "$line" | awk '{print $4}')
        
        if [[ $usage -gt 80 ]]; then
            echo "⚠️  $mount: $used/$size ($usage%) - $avail free"
        else
            echo "✅ $mount: $used/$size ($usage%) - $avail free"
        fi
    done
    echo
    
    # Check for disk thrashing indicators
    local pswpin=$(cat /proc/vmstat | grep pswpin | awk '{print $2}')
    local pswpout=$(cat /proc/vmstat | grep pswpout | awk '{print $2}')
    echo "Swap I/O: ${pswpin} pages in, ${pswpout} pages out (lifetime)"
    echo
}

get_process_info() {
    echo "🔥 TOP PROCESSES"
    echo "----------------"
    echo "Memory hogs:"
    ps aux --sort=-%mem | head -4 | tail -3 | while read line; do
        local pid=$(echo "$line" | awk '{print $2}')
        local mem=$(echo "$line" | awk '{print $4}')
        local cmd=$(echo "$line" | awk '{for(i=11;i<=NF;i++) printf "%s ", $i; print ""}' | cut -c1-50)
        echo "  PID $pid: ${mem}% - $cmd"
    done
    echo
    
    echo "CPU hogs:"
    ps aux --sort=-%cpu | head -4 | tail -3 | while read line; do
        local pid=$(echo "$line" | awk '{print $2}')
        local cpu=$(echo "$line" | awk '{print $3}')
        local cmd=$(echo "$line" | awk '{for(i=11;i<=NF;i++) printf "%s ", $i; print ""}' | cut -c1-50)
        echo "  PID $pid: ${cpu}% - $cmd"
    done
    echo
}

get_wsl_info() {
    echo "🐧 WSL STATUS"
    echo "-------------"
    echo "Workload directories:"
    du -sh /home/me/dev /home/me/.cache /home/me/.local 2>/dev/null | while read size dir; do
        echo "  $dir: $size"
    done
    echo
}

check_alerts() {
    echo "🚨 ALERTS"
    echo "---------"
    
    # Memory alert
    local mem_used=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
    if [[ $mem_used -gt 85 ]]; then
        echo "⚠️  HIGH MEMORY USAGE: ${mem_used}%"
    fi
    
    # Swap alert
    local swap_used=$(free | grep Swap | awk '{print int($3/$2 * 100)}')
    if [[ $swap_used -gt 10 ]]; then
        echo "⚠️  SWAP USAGE: ${swap_used}%"
    fi
    
    # Load alert
    local load1=$(cat /proc/loadavg | awk '{print $1}')
    local cpu_count=$(nproc)
    local load_pct=$(echo "$load1 $cpu_count" | awk '{print int($1/$2 * 100)}')
    if [[ $load_pct -gt 80 ]]; then
        echo "⚠️  HIGH CPU LOAD: ${load1} (${load_pct}% of ${cpu_count} cores)"
    fi
    
    # Disk space alert
    df -h | grep -E "/dev/sd[d-i]" | while read line; do
        local usage=$(echo "$line" | awk '{print $5}' | tr -d '%')
        local mount=$(echo "$line" | awk '{print $6}')
        if [[ $usage -gt 90 ]]; then
            echo "⚠️  DISK SPACE: $mount at ${usage}%"
        fi
    done
    
    # Check if we have any alerts
    if [[ $mem_used -le 85 && $swap_used -le 10 && $load_pct -le 80 ]]; then
        echo "✅ All systems normal"
    fi
    echo
}

log_metrics() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local mem_used=$(free | grep Mem | awk '{print int($3/$2 * 100)}')
    local swap_used=$(free | grep Swap | awk '{print int($3/$2 * 100)}')
    local load=$(cat /proc/loadavg | awk '{print $1}')
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    
    echo "$timestamp,MEM:${mem_used}%,SWAP:${swap_used}%,LOAD:$load,CPU:${cpu_usage}%" >> "$LOGFILE"
}

# Trap Ctrl+C to exit gracefully
trap 'echo -e "\n\n📊 Session log saved to: $LOGFILE"; exit 0' SIGINT

# Main monitoring loop
while true; do
    print_header
    get_memory_info
    get_cpu_info
    get_disk_info
    get_process_info
    get_wsl_info
    check_alerts
    
    log_metrics
    
    echo "================================================================"
    echo "Next update in ${INTERVAL}s... (Ctrl+C to exit)"
    
    sleep "$INTERVAL"
done