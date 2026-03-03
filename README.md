<h1 align="center">Squadro-solver</h1>

[![CI](https://github.com/ndeineko/squadro-solver/actions/workflows/main.yml/badge.svg)](https://github.com/ndeineko/squadro-solver/actions/workflows/main.yml)

***Squadro-solver*** is a program designed to fully explore and solve the strategy board game *Squadro*[^1].

For each reachable game state, it determines which player (if any) has a guaranteed path to victory.

## Key findings

*Note : by convention, the board is rotated so that one player starts from the top while the other starts from the left.*

- The *left* player can guarantee a victory from the starting position, even when playing second.
- There are **46199129613** reachable game states in total.
- Assuming *perfect play* from both players :
  - **21681412181** (≈ 46.93%) of these states are winning for the *top* player.
  - **24492844613** (≈ 53.02%) are winning for the *left* player.
  - **24872819** (≈ 0.05%) are drawing (the game will go on indefinitely).

## Setup and usage

*Note : for a setup-free experience, it is also possible to play against squadro-solver [here](https://squadro-solver.netlify.app/), directly in a web browser.*

The main steps for using this program are detailed in the next subsections and can be summarized as follows :

0. Install Rust development tools and test the program.
1. Generate 3 data files containing reachable and winning game states.
2. Play against the computer (and optionally show if it can force a win).

### Step 0 : prerequisites

#### Install Rust and Cargo

This project uses *Cargo*, Rust's build system and package manager.

[The Rust website](https://rust-lang.org/learn/get-started/) and [the Cargo book](https://doc.rust-lang.org/cargo/getting-started/installation.html) both provide instructions for installing Rust and Cargo on all major operating systems.

On Ubuntu, for instance, the following commands install Rust and Cargo via *Rustup* (a program for managing Rust tools and versions), select the stable toolchain, and also ensure that *gcc* is installed :

```
sudo snap install --classic rustup
rustup default stable
sudo apt install -y gcc
```

#### Test

Once Rust and Cargo are installed, this repository can be cloned and *squadro-solver* can be compiled and tested. All tests should pass. Here are the corresponding commands :

```
git clone https://github.com/ndeineko/squadro-solver
cd squadro-solver
cargo test --release
```

### Step 1 : generate data files

*Note : since this step requires a significant amount of memory and several hours of computation, pre-computed files can be downloaded as an alternative. In that case, all three .data files from [this ZIP archive](https://drive.usercontent.google.com/download?id=1SSzEfMQXZ6MSC-NsHhAq9EzarC8EMUuC&export=download&confirm=t) must be extracted into the current directory (i.e., the one returned by the `pwd` command). After that, the rest of this section can be ignored and the reader can proceed to [the next step](#step-2--play-against-the-computer).*

This step will generate files containing information about the game states. Each file is a ZIP archive containing a chunked bitset, where each bit set to 1 represents a game state with the desired property. Three files will be created :

- *all_states.data* : states reachable by following the game rules.
- *player_0_wins.data* : winning states for the *top* player.
- *player_1_wins.data* : winning states for the *left* player.

Generating the data files requires about **21 GiB** of **free RAM** and 3.2 GiB of free disk space.

<details>
  <summary>
    <i>Note : with memory compression enabled, the required RAM can be reduced to about 11 GiB, for example by using the zram Linux kernel module as shown here.</i>
  </summary>
  The following commands will create a 22 GiB compressed block device in RAM and use it as swap space. Thanks to memory compression, the data files can be successfully generated on a virtual machine running Ubuntu Server 24.04 LTS with 11 GiB of RAM. However, due to the additional time involved in (de)compression operations, it is recommended not to generate the data files with less RAM.

  ```sh
  sudo modprobe zram # enable zram module
  sudo swapoff /dev/zram* # disable existing zram swap spaces
  n=$(sudo cat /sys/class/zram-control/hot_add) # get number of new zram device
  sudo zramctl /dev/zram$n --algorithm zstd --size 22G # create 22 GiB zram device with Zstd compression
  sudo mkswap /dev/zram$n # create swap space inside zram device
  sudo swapon --priority 32767 /dev/zram$n # enable zram swap space
  ```

</details>

The command to generate those files is :

```
cargo run --release -- generate
```

It should complete in 1 to 3 days depending on the system's specifications. Here is the expected output :

    Generating states. This will take a while.
    46199129613 explored states saved.
    Iteration 1 ... Found 19256393173 new winning states for player 0 and 21022870073 for player 1.
    Iteration 2 ... Found 1354248508 new winning states for player 0 and 1689128553 for player 1.
    Iteration 3 ... Found 597720122 new winning states for player 0 and 808920679 for player 1.
    Iteration 4 ... Found 281683757 new winning states for player 0 and 441249020 for player 1.
    Iteration 5 ... Found 116280713 new winning states for player 0 and 285252465 for player 1.
    Iteration 6 ... Found 53887124 new winning states for player 0 and 165225623 for player 1.
    Iteration 7 ... Found 18430182 new winning states for player 0 and 66786892 for player 1.
    Iteration 8 ... Found 2695242 new winning states for player 0 and 13193146 for player 1.
    Iteration 9 ... Found 73360 new winning states for player 0 and 218020 for player 1.
    Iteration 10 ... Found 0 new winning states for player 0 and 142 for player 1.
    Iteration 11 ... Found 0 new winning states for player 0 and 0 for player 1.
    21681412181 winning states saved for player 0.
    24492844613 winning states saved for player 1.

### Step 2 : play against the computer

Once the data files are generated, it is possible to play against the computer.

*Note : the program's strategy is simply to randomly choose a move in the following order of availability : winning moves, drawing moves and losing moves. [The online version](https://squadro-solver.netlify.app/) follows the same logic. Since the left player can always force a win from the starting position, a human victory is only possible if the computer controls the top player.*

The basic command to start a game is :

```
cargo run --release -- play
```

By default, the computer plays against itself from the starting position. To change this behavior, additional arguments can be appended to that command. Some examples are provided below.

- Specify the human-controlled player (*top* or *left*) :
    
    ```
    cargo run --release -- play --player top
    ```
    ```
    cargo run --release -- play --player left
    ```
- Start from a specific position ([the next section](#conversion-between-game-state-and-id) provides instructions for obtaining the ID) :

    ```
    cargo run --release -- play --id 12345
    ```

All arguments and their description can be printed with :

```
cargo run --release -- play --help
```

## Conversion between game state and ID

The game state includes the positions of all pieces as well as the next player to move. This state can be converted into its numerical representation (its *ID*) using one of the two mathematically equivalent formulas below.

<pre>ID = ((((((((((t<sub>0</sub> × 12) + l<sub>0</sub>) × 12 + t<sub>1</sub>) × 12 + l<sub>1</sub>) × 11 + t<sub>2</sub>) × 11 + l<sub>2</sub>) × 12 + t<sub>3</sub>) × 12 + l<sub>3</sub>) × 12 + t<sub>4</sub>) × 12 + l<sub>4</sub>) × 2 + p</pre>
<pre>ID = p + 2 l<sub>4</sub> + 24 t<sub>4</sub> + 288 l<sub>3</sub> + 3456 t<sub>3</sub> + 41472 l<sub>2</sub> + 456192 t<sub>2</sub> + 5018112 l<sub>1</sub> + 60217344 t<sub>1</sub> + 722608128 l<sub>0</sub> + 8671297536 t<sub>0</sub></pre>

Where :

- Variable **p** represent the player whose turn it is to play. Its value is *0* for the *top* player and *1* for the *left* player.
- Variables **t<sub>0</sub>** to **t<sub>4</sub>** represent the positions of the *top* player's pieces. The values of those five variables are given in the next table for all reachable positions. The first and second numbers indicate the values ​​that apply when the piece is pointing down and up, respectively.
    <table>
      <tbody>
        <tr>
          <th scope="col" align="center">t<sub>0</sub></th>
          <th scope="col" align="center">t<sub>1</sub></th>
          <th scope="col" align="center">t<sub>2</sub></th>
          <th scope="col" align="center">t<sub>3</sub></th>
          <th scope="col" align="center">t<sub>4</sub></th>
        </tr>
        <tr>
          <td align="center">0 / 11</td>
          <td align="center">0 / 11</td>
          <td align="center">0 / 10</td>
          <td align="center">0 / 11</td>
          <td align="center">0 / 11</td>
        </tr>
        <tr>
          <td align="center">1 / 10</td>
          <td align="center">‒ / 10</td>
          <td align="center">‒ / 9</td>
          <td align="center">‒ / 10</td>
          <td align="center">1 / 10</td>
        </tr>
        <tr>
          <td align="center">2 / 9</td>
          <td align="center">1 / 9</td>
          <td align="center">1 / 8</td>
          <td align="center">1 / 9</td>
          <td align="center">2 / 9</td>
        </tr>
        <tr>
          <td align="center">3 / 8</td>
          <td align="center">2 / 8</td>
          <td align="center">2 / 7</td>
          <td align="center">2 / 8</td>
          <td align="center">3 / 8</td>
        </tr>
        <tr>
          <td align="center">4 / 7</td>
          <td align="center">3 / 7</td>
          <td align="center">3 / 6</td>
          <td align="center">3 / 7</td>
          <td align="center">4 / 7</td>
        </tr>
        <tr>
          <td align="center">5 / ‒</td>
          <td align="center">4 / 6</td>
          <td align="center">4 / ‒</td>
          <td align="center">4 / 6</td>
          <td align="center">5 / ‒</td>
        </tr>
        <tr>
          <td align="center">‒ / 6</td>
          <td align="center">‒ / 5</td>
          <td align="center">‒ / 5</td>
          <td align="center">‒ / 5</td>
          <td align="center">‒ / 6</td>
        </tr>
      </tbody>
    </table>
- Variables **l<sub>0</sub>** to **l<sub>4</sub>** represent the positions of the *left* player's pieces. The values of those five variables are given in the next table for all reachable positions. The first and second numbers indicate the values ​​that apply when the piece is pointing right and left, respectively.
    <table>
      <tbody>
        <tr>
          <th scope="row" align="center">l<sub>0</sub></th>
          <td align="center">0 / 11</td>
          <td align="center">‒ / 10</td>
          <td align="center">1 / 9</td>
          <td align="center">2 / 8</td>
          <td align="center">3 / 7</td>
          <td align="center">4 / 6</td>
          <td align="center">‒ / 5</td>
        </tr>
        <tr>
          <th scope="row" align="center">l<sub>1</sub></th>
          <td align="center">0 / 11</td>
          <td align="center">1 / 10</td>
          <td align="center">2 / 9</td>
          <td align="center">3 / 8</td>
          <td align="center">4 / 7</td>
          <td align="center">5 / ‒</td>
          <td align="center">‒ / 6</td>
        </tr>
        <tr>
          <th scope="row" align="center">l<sub>2</sub></th>
          <td align="center">0 / 10</td>
          <td align="center">‒ / 9</td>
          <td align="center">1 / 8</td>
          <td align="center">2 / 7</td>
          <td align="center">3 / 6</td>
          <td align="center">4 / ‒</td>
          <td align="center">‒ / 5</td>
        </tr>
        <tr>
          <th scope="row" align="center">l<sub>3</sub></th>
          <td align="center">0 / 11</td>
          <td align="center">1 / 10</td>
          <td align="center">2 / 9</td>
          <td align="center">3 / 8</td>
          <td align="center">4 / 7</td>
          <td align="center">5 / ‒</td>
          <td align="center">‒ / 6</td>
        </tr>
        <tr>
          <th scope="row" align="center">l<sub>4</sub></th>
          <td align="center">0 / 11</td>
          <td align="center">‒ / 10</td>
          <td align="center">1 / 9</td>
          <td align="center">2 / 8</td>
          <td align="center">3 / 7</td>
          <td align="center">4 / 6</td>
          <td align="center">‒ / 5</td>
        </tr>
      </tbody>
    </table>

## License

This project is licensed under [the MIT License](LICENSE).

[^1]: Squadro is a 2-player board game created by Adrian Jimenez Pascual (https://dirdam.github.io/squadro.html) and published by GIGAMIC (https://en.gigamic.com/modern-classics/504-squadro.html).
