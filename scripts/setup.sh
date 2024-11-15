. ../config/config.config

# Make services
sudo cp services/*.service /etc/systemd/system/

# Make SurrealDB service
sudo adduser --system --no-create-home --group surrealdb
sudo mkdir /etc/surrealdb
sudo chown -R surrealdb:surrealdb /etc/surrealdb
sudo systemctl daemon-reload
sudo systemctl enable surrealdb.service

# Make Rust program service
sudo adduser --system --no-create-home --group rustapp
sudo chown -R rustapp:rustapp BeatmapBrowser/site/output
sudo cp services/rustapp.service /etc/systemd/system/rustapp.service
sudo systemctl daemon-reload
sudo systemctl enable rustapp.service

# Setup certs
sudo cp ../config/certs/site.crt /etc/ssl/certs/site.crt
sudo chown root:root /etc/ssl/certs/site.crt
sudo cp ../config/privatae/site.key /etc/ssl/private/site.key
sudo chown root:root /etc/ssl/private/site.key
sudo chmod 600 /etc/ssl/private/site.key

# Setup site
cp ./config/sites-available/*.conf /etc/nginx/sites-available/
sed -i -e "s/{DOMAIN}/$DOMAIN/g" /etc/nginx/sites-available/*.conf

# Setup nginx
sudo rm /etc/nginx/sites-enabled/default
sudo ln -s /etc/nginx/sites-available/www.site /etc/nginx/sites-enabled/
sudo systemctl reload nginx
sudo ufw allow 'Nginx Full'