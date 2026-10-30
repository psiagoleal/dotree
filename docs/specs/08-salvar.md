<!-- Caminho relativo: docs/specs/08-salvar.md -->

# 08 — Salvar, abrir e retomar

## Resultado

O usuário não perde o exercício ao fechar a aba e pode guardar ou trocar um exercício como
arquivo.

## Critérios

- **SAL-01** — O sistema DEVE salvar o exercício em andamento no armazenamento local do
  navegador a cada mudança e oferecer a retomada na visita seguinte.
  *Prova:* teste de ponta a ponta (recarrega a página no meio da compressão).
- **SAL-02** — O sistema DEVE exportar o exercício como arquivo JSON com versão de esquema,
  contendo expressão, larguras, células por nível e registro de ajudas.
  *Prova:* teste do núcleo (exportar → importar devolve o mesmo estado).
- **SAL-03** — SE o arquivo importado for inválido ou de versão de esquema desconhecida, ENTÃO
  o sistema DEVE recusá-lo explicando o motivo, sem alterar o exercício aberto.
  *Prova:* testes do núcleo com JSON malformado, esquema futuro e célula inconsistente.
- **SAL-04** — SE o armazenamento local estiver indisponível (navegação privada, bloqueio),
  ENTÃO o sistema DEVE funcionar normalmente e avisar uma vez que o progresso não será
  lembrado.
  *Prova:* teste de ponta a ponta com o armazenamento bloqueado.
- **D-6 ⚠ A CONFIRMAR** — compartilhamento por link (exercício codificado na URL) fica fora da
  v0.1.

## Fora de escopo

Sincronização entre dispositivos; histórico de versões; contas.
