#!/bin/bash

tmux new-session -s fh -d
tmux rename-window -t fh main

tmux send-keys -t fh 'just run' Enter
tmux split-window -h -t fh
tmux select-pane -t 1
tmux split-window -v -t fh
tmux send-keys -t fh 'just run-gateway' Enter
tmux select-pane -t 3
tmux attach -t fh
