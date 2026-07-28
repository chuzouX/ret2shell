# Docker Compose Deployment

Deployment example using [Docker Compose](https://docs.docker.com/compose/).

> [!NOTE]
>
> The provided [docker-compose.yml](docker-compose.yml) file is fully configured with all services required to run the application.
>
> If you want to start server binary independently, you can comment out the `platform` and `nginx` service in the compose file and expose ports under other services as the comments suggest.

Directory [config/](config/) must contain `config.toml` (mount to `/etc/ret2shell/` as stated at [/crates/config/src/lib.rs](../../crates//config/src/lib.rs)) for the `platform` service. You need to change the configuration file (example given at [/config/config.sample.toml](../../config/config.sample.toml)) to fit your compose configuration. For further description of this directory, please refer to [config/README.md](config/README.md).

Directory [static/](static/) contains frontend static files (mount to `/var/www/html/`), for `nginx` service.

Put other configuration files, such as the sensitive-word list, in the [config/](config/) directory.
