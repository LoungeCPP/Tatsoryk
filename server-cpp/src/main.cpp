#include <iostream>
#include <string>

void listen(std::string host, int port) {
    std::cout << "Listening on " << host << ":" << port << "\n";
    // TODO actually listen
}

int main(int argc, char** argv) {
    std::string host = "127.0.0.1";
    int port = 8080;

    if (argc >= 2) {
        host = std::string(argv[1]);
    }

    if (argc >= 3) {
        port = std::stoi(argv[2]);
    }

    listen(host, port);
    return 0;
}
