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
  contendo expressão, larguras, células por nível, revelações já feitas e registro de ajudas.
  *Prova:* teste do núcleo (exportar → importar devolve o mesmo estado).
- **SAL-03** — SE o arquivo importado for inválido ou de versão de esquema desconhecida, ENTÃO
  o sistema DEVE recusá-lo explicando o motivo, sem alterar o exercício aberto.
  *Prova:* testes do núcleo com JSON malformado, esquema futuro e célula inconsistente.
- **SAL-04** — SE o armazenamento local estiver indisponível ou cheio, ENTÃO o sistema DEVE
  funcionar normalmente e avisar uma vez que o progresso não será lembrado.
  *Prova:* teste de ponta a ponta com o armazenamento bloqueado.
- **SAL-05** — SE o estado salvo localmente estiver corrompido ou for de esquema antigo
  incompatível, ENTÃO o sistema DEVE abrir sem ele, avisar e oferecer baixar o conteúdo
  salvo como arquivo antes de descartá-lo.
  *Prova:* teste de ponta a ponta com estado salvo adulterado.

## Notas de implementação

- **Formato (esquema 1, MT-15a).** Objeto JSON com `format: "dotree-exercise"`, `schema`,
  `expression`, `widths`, `levels` (cada um com `cpa` e `cells`), `pending` (células do
  nível atual, na ordem em que foram formadas), `reveals.width` (resposta dada, ou `null`
  para quem desistiu, e se a conta foi aberta) e `help` (o registro de ajudas, AJU-07).
  Cada célula guarda `kind`, `column` e `inputs`. As entradas são **posições na ordem
  canônica dos pontos do nível**, e não ids internos, porque voltar de nível gera ids
  novos. O nível do CPA não lista células: elas decorrem dos pontos do nível.
- **Revelações.** O limite inferior e a solução revelados (etapa 1 ou 2) decorrem do
  registro de ajudas, por isso não se repetem em `reveals`. Ao reabrir, a tela mostra de
  novo tudo o que já foi revelado sem pedir confirmação nem contar outra vez. Dica e
  próxima jogada valem só para o estado em que foram pedidas, então ficam apenas na
  contagem.
- **Abrir = refazer.** O núcleo refaz cada célula pelas regras da compressão, e qualquer
  recusa vira "célula inconsistente", com nível e posição. O histórico de desfazer volta
  como a sequência das células salvas. Arquivos acima de 1 MB são recusados sem leitura.

## Fora de escopo

Compartilhamento por link (D-6); sincronização entre dispositivos; histórico de versões;
contas.
