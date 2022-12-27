# tic-tac-toe

É um jogo da velha escrito em Rust usando gRPC.
A arqutetura do projeto é baseada em um servidor principal que cria salas (permitindo até dois jogadores por sala) e processa as jogadas dos jogadores, lhes entregando o tabuleiro atualizado, após a vitoria do primeiro jogador, a sala é encerrada.
