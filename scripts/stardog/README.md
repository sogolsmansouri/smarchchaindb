# Stardog

Follow steps to install [Stardog](https://www.stardog.com) on [Docker](https://www.docker.com)

This setup is intended for [VCL](https://vcl.ncsu.edu)

## Setup

1. Install docker [instructions](https://docs.docker.com/install/linux/docker-ce/ubuntu/)

2. Clone the bigchaindb repo

3. Enter the scripts directory

4. Download the stardog license and place it in this directory

5. Build the image using the following command

    ```sh
    docker build -t 'stardog' .
    ```

6. Start the container using

    ```sh
    docker container run --name stardog -p 5820:5820 -d -e STARDOG_SERVER_FLAGS="--web-console" --restart always stardog
    ```

    *You can ignore the `STARDOG_SERVER_FLAGS` environment variable if you don't want to enable the web-console*