# tic-tac-toe

É um jogo da velha escrito em Rust usando gRPC.

A arqutetura do projeto é baseada em um servidor principal que cria salas, processando as jogadas dos jogadores, lhes entregando o tabuleiro atualizado e o estado da partida, informando se a partida já possui um vencedor ou não.

O servidor não é muito restritivo (ainda), os detalhes a cerca da "jogabilidade" do jogo da velha estão implementadas no client.

## Como jogar

### Iniciando o servidor

Inicie o servidor em uma sessão do seu terminal.

```
cargo run --bin server
```
Você deverá ver o seguinte log, informando que o server está executando no localhost, na porta `50051`:
```
❯ cargo run --bin server
    Finished dev [unoptimized + debuginfo] target(s) in 0.24s
     Running `target/debug/server`
Serving on [::1]:50051
```

### Criando uma partida com o client

Em outra sessão, inicie o client, ele recebe os seguites argumentos: `<shape> <optional| room_id>` (com o room id é opcional, se não for passado o cliente iniciará o jogo criando uma nova sala).

Geralmente jogo da velha se joga como "X" ou "O" aqui eu informo que quero jogar nesta sessão como "X":

```  
cargo run --bin client -- X
```

Você verá a primeira (e única) tela do client:

<img width="753" alt="image" src="https://user-images.githubusercontent.com/22692488/209725116-0bbf324d-08f1-419a-83dd-72909ff35ac1.png">

Copie o **room id** , ele é o identificador que será usado para outro jogador poder entrar na sua sala!!

### Entrando em uma sala para jogar

Com o **room id**  em mãos, execute em outra sessão:

```
cargo run --bin client -- O 5870d06f-aa49-452f-be12-c2a77975f49e
```

O Client iniciará entrando na sala informada.

### Jogando

As ações no tabuleiro são feitas através da emissão de coordenadas, ou seja, passar `0 0` diz que você quer marcar na linha 0, posição 0. Semelhante a como se faz ações no jogo batalha naval.

As telas do client são atualizadas assincronamente, ou seja, uma ação feita pelo jogador 1 é visualizada na tela do jogador 2 instantaneamente!

Uma vez que haja um vencedor, o client encerra a partida:
<img width="753" alt="image" src="https://user-images.githubusercontent.com/22692488/209725826-45f75f75-b2e4-4d58-a9fc-af82c103207d.png">


## Tech Stack

O servidor é construído no topo das bibliotecas `tokio`, `tonic` e `prost` para servir uma aplicação gRPC, o método rpc `WatchRoomStream` é um server stream, entregando o último estado do board através de uma **stream infinita**.

O Client é feito usando a biblioteca [tui-rs](https://github.com/fdehau/tui-rs), muito da interface do client do tic-tac-toe foi fortemente inspirado nos exemplos do diretório de /examples da tui-rs.

O algoritmo utilizado para processar o jogo da velha é demasiadamente simples, não houve necessidade de se utilizar bibliotecas externas para ele.


## Melhorias mapeadas

Há algumas melhorias a serem concluídas, principalmente por parte do servidor, as mais críticas são: Evitar que jogadores façam jogadas fora do seu turno, encerrar salas concluídas e evitar jogadas repetidas no mesmo lugar.

