package main

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
)

func main() {
	change_env := exec.Command("rooch", "env", "switch", "--alias", "main")
	out, err := change_env.CombinedOutput()
	if err != nil {
		fmt.Printf("combined out:\n%s\n", string(out))
		log.Fatal("change_env.Run() failed with %s\n", err)
	}
	fmt.Printf("combined out\n%s\n", string(out))

  read_file("addresses.txt")
}

func read_file(path string) error {
  f, err := os.OpenFile(path, os.O_RDONLY, 0666)
  if err != nil {
    return err
  }

  defer f.Close()

  reader := bufio.NewReader(f)

  var results []string
  for {
    line, _, err := reader.ReadLine()
    if err == io.EOF {
      break
    }
    // rooch account transfer --to rooch1z6jp5k2vj2hs75zdqe5rhgdaj4w3hjnm693ks6xaxxdr78vvxreqwrrvpt --coin-type 0x3::gas_coin::RGas --amount 1
    send := exec.Command("rooch", "account", "transfer", "--coin-type", "0x3::gas_coin::RGas", "--amount", "1", "--to", string(line))
    _, err = send.CombinedOutput()
    if err != nil {
      // fmt.Printf("combined out:\n%s\n", string(out))
      fmt.Println(string(line), "=> ok")
      // log.Fatal("send.Run() failed with %s\n", err)
      log.Fatal("send failed\n")
    }
      fmt.Println(string(line), "=> ok")
    // fmt.Printf("combined out\n%s\n", string(out))


    // fmt.Println(string(line))
    results = append(results, string(line))
  }
  fmt.Printf("Read results: %v\n", results)
  return nil
}
