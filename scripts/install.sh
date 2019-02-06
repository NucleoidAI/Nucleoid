#!/bin/sh
sudo apt update
sudo apt install nodejs -y
sudo apt install npm -y

read -p "user.name: " name
git config --global user.name "$name"
read -p "user.email: " email
git config --global user.email "$email"
