---
outline: deep
---

# System Monitor

The System Monitor provides real-time information about your system performance and resources.

## Overview

The System Monitor updates every 500 milliseconds and displays key metrics in a bento box layout that adapts to your window size.

## Tabs

### Overview

The Overview tab shows key metrics at a glance:

- **CPU Usage**: Current CPU utilization percentage with a sparkline graph showing recent history
- **Memory**: RAM usage percentage and amount used/total
- **Disk**: Primary disk usage percentage and space used/total
- **Network**: Current download speed with upload speed as a sub-value
- **Processes**: Number of active processes
- **Battery**: Battery percentage and status (on laptops)

The layout adapts automatically based on window width:

- **Desktop** (≥1200px): Network card spans the left side with other metrics in a 2x2 grid
- **Tablet** (768-1200px): Two-column layout with network on top
- **Mobile** (<768px): Single column layout

### System

Detailed system information including:

- CPU specifications and per-core usage
- Memory breakdown (used, free, cached, buffers)
- Detailed disk information for all mounted drives
- Process list with CPU and memory usage

### Network

Network interface details:

- List of all network interfaces
- Current upload and download speeds per interface
- Data usage statistics
- Interface status (up/down)

### Disks

Comprehensive disk information:

- All mounted filesystems
- Total, used, and available space
- Usage percentages
- Filesystem types

## Color Coding

Metrics use color to indicate status:

- **Blue/Green**: Normal operation
- **Yellow**: Warning threshold exceeded
- **Red**: Critical threshold exceeded

Thresholds are:

- CPU: Warning at 70%, Critical at 90%
- Memory: Warning at 80%, Critical at 95%
- Disk: Warning at 85%
- Battery: Low at 20%

## Sparklines

Real-time sparkline graphs show the last 60 data points for CPU, memory, and network metrics, giving you a visual history of system performance.

## Quick Access

You can open the System Monitor directly from the command line:

```bash
omarchist --view system
```
