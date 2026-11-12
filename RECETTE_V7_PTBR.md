# Recycling System Industrial 7.0.0 — Roteiro de homologação

## Atualização

1. Feche a aplicação e faça uma cópia do banco `lxo.sqlite` e de seus arquivos `-wal`/`-shm` (se existirem). A base **não** deve ser excluída na atualização.
2. Copie somente a **chave pública** usada anteriormente para `src-tauri/src/license_public_key.hex`. Não regenere as chaves nem inclua a chave privada no aplicativo.
3. Na pasta `desktop`, execute `npm install`, `npm test` e `npm run tauri:dev`. Compile o AppImage apenas após os testes na máquina Linux.
4. Confirme que a versão exibida no rodapé é `7.0.0`.

## Casos de teste

| Nº | Tela | Procedimento | Resultado esperado |
|---|---|---|---|
| 1 | Pesagem | Entre em Pesagem/F1 | Foco automático em Código ou nome do produto |
| 2 | Pesagem | Digite `FERRO` (ou outro código cadastrado), Enter | Produto selecionado; foco no Peso |
| 3 | Pesagem | Digite `12,500`, Enter | Foco no preço unitário, valor cadastrado visível ao lado do peso |
| 4 | Pesagem | Enter no preço | Item adicionado com nome, peso, R$/kg e total; campos limpos; foco volta ao produto |
| 5 | Pesagem | Digite início do nome em vez de código | Lista continua filtrando pelos nomes |
| 6 | Configuração | Produto com “Permitir alterar preço/kg” marcado | Preço editável durante Pesagem |
| 7 | Configuração | Produto sem a permissão | Preço visível, porém bloqueado; Enter ainda registra o valor cadastrado |
| 8 | Caixa | Sem caixa aberto, clique Abrir Caixa | Botão maior e diálogo de abertura |
| 9 | Caixa | Digite `12,78` sem apagar zeros | Campo aceita digitação; ao sair, exibe `12,78` |
| 10 | Caixa | Registre compra de dois materiais | Linha do tempo exibe a operação e seu horário/valor total |
| 11 | Caixa | Clique “Ver produtos, pesos e valores” | Detalhe apresenta todos os materiais, pesos, R$/kg e totais |
| 12 | Caixa | Clique Fechar Caixa | Botão maior e confirmação |
| 13 | Configuração/Controle | Admin: clique “Fazer backup e zerar dados financeiros” | Dupla confirmação e backup SQLite prévio |
| 14 | Configuração/Controle | Confirme `ZERAR FINANCEIRO` | Compras, vendas, pagamentos, caixas, históricos operacionais e despesas zerados |
| 15 | Configuração | Confira produtos/categorias, usuários, clientes e balanças | Cadastros e parâmetros preservados |
| 16 | Licenciamento | Reinicie o aplicativo | Licença existente continua válida |
| 17 | Login Funcionário | Acesse com perfil Funcionário | Apenas Pesagem disponível; manutenção e finanças bloqueadas |
| 18 | Resiliência | Cancele um dos dois diálogos de limpeza | Banco permanece intacto |

## Observações de segurança

O botão de limpeza é para homologação, está disponível apenas ao **Administrador** em Configuração → Controle e exige uma frase de confirmação. A API também verifica o perfil antes de aceitar a operação. O programa grava um arquivo de backup `lxo-pre-reset-financeiro-*.sqlite` no mesmo diretório do banco antes de executar a exclusão transacional. Se a cópia de segurança falhar, nenhuma tabela é apagada.

Os cadastros não financeiros (produtos, categorias, usuários, clientes, balanças e configurações) são preservados. Categorias de despesas permanecem cadastradas, mas lançamentos de despesa são apagados. O arquivo `.lic` e o local configurado para a licença não são alterados.

**Atenção:** os testes JavaScript inspecionam a implementação; a compilação Rust/Tauri e a homologação com uma base real devem ser executadas na máquina de implantação.
