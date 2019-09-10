#!/bin/sh
sudo apt update
sudo apt install nodejs -y
sudo apt install npm -y
sudo apt install aspell-en -y

read -p "user.name: " name
git config --global user.name "$name"
read -p "user.email: " email
git config --global user.email "$email"

sudo npm install prettier -g
sudo npm install eslint -g
sudo npm install mocha -g
