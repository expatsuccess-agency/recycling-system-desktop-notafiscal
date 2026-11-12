# Recycling System Industrial — V7.1.0 — Guia de testes

**Antes de atualizar:** feche o programa e faça uma cópia de `~/.local/share/br.com.lxo.desktop/lxo.sqlite`. Preserve seu arquivo `.lic` e a chave pública Ed25519 já utilizada; não gere novas chaves.

## Checklist de verificação

1. **Pesagem:** abra F1. O seletor Código ou nome deve permanecer fechado quando receber foco. Digite `FER`, confirme com Enter e verifique a passagem para Peso.
2. **Campos:** Peso (kg) e Preço unitário (R$/kg) devem ter largura e altura equivalentes. Continue Enter → Peso → Preço/kg → novo Código.
3. **Menu:** o item F2 chama-se **Controle**, preservando a tela de vendas; Gestão Financeira aparece antes de Configuração.
4. **Ícone:** em Configuração → Produtos e Categorias, confira fundo branco e tamanho reduzido do seletor.
5. **Abas:** em Configuração veja Produtos e Categorias, Usuários e Senhas, Controle, Manutenção do banco. Usuários aparece somente na própria aba.
6. **Caixa do funcionário:** faça login como Funcionário e verifique F3, F6, F7 e os recursos de Caixa. Vendas, Finanças e Configuração permanecem bloqueados.
7. **F10:** como administrador, abra Controle (F2), insira itens e pressione F10 para abrir a confirmação do recebimento.
8. **Avisos:** tente autenticar com senha incorreta. Deve aparecer texto amigável em português, não `CREDENCIAIS_INVÁLIDAS`.
9. **Manutenção (SOMENTE EM BANCO DE TESTES):** em Configuração → Manutenção do banco, marque a confirmação, digite `ZERAR FINANCEIRO` e execute. Observe Registro detalhado da operação. Verifique o caminho do backup retornado.
10. **Preservação:** após a limpeza, confira que caixas, compras, vendas e despesas foram zerados e que produtos, usuários, categorias e licença permaneceram intactos. Caso falhe, copie o registro e a mensagem técnica exibida.

### Instalação

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V7.1/recycling-system-v7/desktop
bash sync-license-key.sh
npm install
npm test
npm run tauri:dev
```

O script `sync-license-key.sh` deve localizar sua chave pública existente; nunca inclua `private-key.pem` no cliente.
