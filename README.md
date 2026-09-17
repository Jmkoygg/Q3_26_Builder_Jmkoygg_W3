# Turbin3 Q3 2026 - Week 3 Assignment: Automated Market Maker (AMM)

Repositório da entrega das tarefas obrigatórias da **Semana 3 (AMMs)** do programa **Turbin3 Builders**.

Este projeto implementa um **Automated Market Maker (AMM)** completo na blockchain Solana utilizando o framework **Anchor**, com matemática de produto constante ($X \times Y = K$), suporte a taxas de protocolo roteadas para uma conta de **Tesouraria (Treasury PDA)**, mecanismo de pausa/bloqueio para emergências e uma suíte de testes automatizados com **LiteSVM**.

---

## 📋 Resumo das Tarefas Obrigatórias

| Tarefa | Descrição | Status |
|---|---|---|
| **1. Write the AMM program** | Implementação das instruções fundamentais (`initialize`, `deposit`, `withdraw`, `swap` e `lock`/`unlock`). | ✅ Concluído |
| **2. Add fees and treasury account** | Cálculo de taxas de negociação (em *basis points*) e direcionamento automático para contas de token da Tesouraria governadas por PDA, além da instrução `withdraw_fees`. | ✅ Concluído |
| **3. Write tests covering all the instructions** | Suíte de testes com `LiteSVM` cobrindo 100% dos fluxos e instruções com execução local instantânea (0.66s). | ✅ Concluído |

---

## 🏛️ Arquitetura e Contas

Na Solana, programas não armazenam dados internamente; tudo reside em **Contas (PDAs)**. Nosso AMM utiliza a seguinte estrutura:

```
                  ┌──────────────────────────────────┐
                  │          Initializer             │
                  └────────────────┬─────────────────┘
                                   │ init
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                          Pool Config (PDA)                             │
│                  seeds = [b"config", seed.to_le_bytes()]               │
│  - fee: u16 (bps)                                                      │
│  - locked: bool                                                        │
│  - authority: Option<Pubkey>                                           │
└──────┬───────────────────────┬─────────────────────────┬───────────────┘
       │ authority             │ authority               │ mint authority
       ▼                       ▼                         ▼
┌──────────────┐        ┌──────────────┐         ┌──────────────┐
│   Vault X    │        │   Vault Y    │         │  LP Mint     │
│   (ATA)      │        │   (ATA)      │         │    (PDA)     │
└──────────────┘        └──────────────┘         └──────────────┘

                                   │ vinculada
                                   ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        Treasury Authority (PDA)                        │
│                  seeds = [b"treasury", config.key()]                   │
└──────────────────────┬─────────────────────────────────────────────────┘
                       │ authority
          ┌────────────┴────────────┐
          ▼                         ▼
   ┌──────────────┐          ┌──────────────┐
   │  Treasury X  │          │  Treasury Y  │
   │    (ATA)     │          │    (ATA)     │
   └──────────────┘          └──────────────┘
```

### Contas Principais
1. **`config` (PDA)**: Guarda os parâmetros do pool (mints X e Y, taxa, status de bloqueio, autoridade e bumps).
2. **`vault_x` e `vault_y` (ATAs)**: Contas de token onde a liquidez do pool fica custodiada. A autoridade dessas contas é o PDA `config`.
3. **`mint_lp` (Mint PDA)**: Token SPL emitido para os provedores de liquidez proporcionalmente ao depósito.
4. **`treasury` (PDA)**: Autoridade do protocolo para recebimento de taxas, sem chave privada.
5. **`treasury_x` e `treasury_y` (ATAs)**: Contas de custódia das taxas arrecadadas em Token X e Token Y, controladas pelo PDA `treasury`.

---

## ⚙️ Fluxo das Instruções

### 1. `initialize`
- Inicializa a conta `config` com a taxa desejada (ex: `30` bps = 0.3%).
- Cria a mint de LP tokens e as duas vaults (`vault_x` e `vault_y`).
- Inicializa as contas associadas da tesouraria (`treasury_x` e `treasury_y`).

### 2. `deposit`
- Permite que Provedores de Liquidez (LPs) depositem Token X e Token Y na proporção da curva.
- Valida que o pool não está bloqueado (`!config.locked`).
- Emite (`mint_to`) LP tokens correspondentes à cota do usuário.

### 3. `swap`
- Executa a troca de tokens respeitando o modelo **$X \times Y = K$**:
  1. A curva calcula a taxa com base no montante de entrada: $\text{fee} = \text{amount} \times \frac{\text{fee\_bps}}{10.000}$.
  2. O depósito líquido ($\text{amount} - \text{fee}$) vai para a respectiva **Vault do Pool**.
  3. O valor da taxa ($\text{fee}$) vai diretamente para a respectiva conta da **Tesouraria** (`treasury_x` ou `treasury_y`).
  4. O usuário recebe os tokens de saída da vault oposta, com proteção contra slippage (`min_amount_out`).

### 4. `withdraw`
- O LP queima (`burn`) seus LP tokens.
- O programa calcula a proporção devida e devolve os tokens X e Y das vaults.

### 5. `withdraw_fees`
- Permite que a autoridade do pool (`authority`) realize o resgate das taxas acumuladas na tesouraria.
- O programa assina a transferência via CPI utilizando as sementes do PDA da tesouraria (`treasury_seeds`).

### 6. `lock` & `unlock`
- Mecanismo de segurança ("circuit breaker") que permite à autoridade pausar ou despausar o pool.
- Impede `deposit`, `withdraw` e `swap` enquanto o pool estiver com status `locked = true`.

---

## 🧪 Suíte de Testes (LiteSVM)

Os testes foram construídos em Rust utilizando **`LiteSVM`**, executando o runtime da Solana diretamente em memória sem necessidade de inicializar um `solana-test-validator` pesado.

### Como Executar os Testes

```bash
cargo test
```

### Saída dos Testes

```text
running 6 tests
test test_initialize ... ok
test test_deposit ... ok
test test_swap_and_fee_to_treasury ... ok
test test_withdraw ... ok
test test_withdraw_fees ... ok
test test_lock_and_unlock_pool ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
```

### O Que Cada Teste Valida:
1. **`test_initialize`**: Garante a criação correta de todas as contas, bumps, vaults e contas de tesouraria.
2. **`test_deposit`**: Valida a transferência de tokens para as vaults e a emissão dos LP tokens.
3. **`test_swap_and_fee_to_treasury`**: Realiza um swap de 10 tokens com 0.3% de taxa, conferindo na ponta do lápis que exatamente 30.000 unidades foram para a Tesouraria e o restante foi para a vault.
4. **`test_withdraw`**: Testa o resgate de liquidez com a queima do recibo LP.
5. **`test_withdraw_fees`**: Valida que o admin consegue sacar as taxas acumuladas da tesouraria para sua carteira.
6. **`test_lock_and_unlock_pool`**: Comprova que operações de swap falham com o erro `PoolLocked` quando o pool é pausado, e voltam a funcionar normalmente após o desbloqueio.

---

## 💡 Guia Rápido para Apresentação (Perguntas da Banca)

Se o mentor ou professor perguntar durante a avaliação:

- **Por que usamos um PDA para a Tesouraria em vez de uma carteira normal?**
  > *Porque um PDA é determinístico e governado pelo código do programa. Ninguém tem a chave privada; apenas o programa pode autorizar saques através da instrução `withdraw_fees` respeitando as regras de autoridade.*

- **Onde foi calculada a taxa de swap?**
  > *A taxa é calculada em basis points (30 bps = 0.3%). Durante a instrução `swap`, o valor total de entrada é decomposto: o montante líquido alimenta a vault para preservar a curva $K = X \times Y$, e a taxa é enviada imediatamente para a conta `treasury_x` ou `treasury_y`.*

- **Por que usamos `LiteSVM` em vez de `anchor test` com TypeScript?**
  > *O `LiteSVM` roda direto no runtime Rust do processo de testes. É centenas de vezes mais rápido (0.66 segundos para 6 testes completos), totalmente determinístico e elimina a complexidade de instanciar validadores externos.*
