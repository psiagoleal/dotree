<!-- Caminho relativo: docs/adr/0004-spec-viva-e-referencias-publicas.md -->

# ADR 0004: Uma spec só, viva, e só referências públicas

- **Status:** Accepted
- **Data:** 2026-11-10
- **Decisores:** Iago Leal
- **Tags:** documentação, spec, referências

## Contexto

A spec em `docs/specs/` nasceu como o contrato da v0.1 e foi crescendo ticket a ticket
(constante aditiva, Wallace, comparação, entrada por alturas). Antes de fechar a v0.2, era
preciso decidir se cada versão ganha a sua própria spec ou se a mesma spec acompanha o
projeto.

O site no GitHub Pages é público: quem o abre não é só uma turma, é qualquer pessoa. Ao
longo do desenvolvimento, alguns textos do repositório citavam, para explicar uma escolha,
material didático que não é público (listas de exercício, soluções, roteiros de
laboratório, slides). Quem lê o repositório não consegue conferir essas fontes, e citá-las
expõe material de terceiros.

## Alternativas consideradas

| Alternativa | Por que não |
|---|---|
| **Uma spec por versão** (`specs/v0.1/`, `specs/v0.2/`…) | Duplica o que não mudou; um critério corrigido teria de ser corrigido em várias cópias, e a cópia velha ainda pareceria valer. O git já guarda cada versão. |
| **Spec congelada na v0.1 e notas de mudança à parte** | O contrato deixaria de descrever o app; os testes citariam critérios que não existem em lugar nenhum. |
| **Citar material não público "só pela referência"** (número do problema, nome do roteiro) | Não pode ser conferido por quem lê, e ainda identifica o material. |

## Decisão

- **A spec é uma só e viva.** `docs/specs/` descreve o que o `main` faz. Cada ticket que
  muda comportamento muda a spec no mesmo PR (regra de manutenção em
  `docs/specs/README.md`). O estado de cada versão fica no git (a *tag*) e o que mudou,
  no `CHANGELOG.md`. Identificadores de critério não são renumerados.
- **O escopo fora da versão atual** fica numa lista só, na spec 00, e cada item aponta
  para o ticket que o trataria.
- **Só se citam livros e artigos de acesso público**: B. Parhami, *Computer Arithmetic:
  Algorithms and Hardware Designs*; C. S. Wallace (1964); L. Dadda (1965); e os artigos
  das técnicas que o app implementar. Material didático não publicado não é citado nem
  por referência. Casos e exemplos usados em teste são do próprio projeto e se apresentam
  como tal, conferidos à mão e, quando possível, por um cálculo independente.

## Consequências

- **Impacto positivo:** um lugar só para saber o que o app faz; todo número citado pode
  ser conferido por quem lê; o repositório pode ser mostrado a qualquer pessoa.
- **Impacto negativo:** para ver o contrato de uma versão antiga é preciso abrir a *tag*
  no git. Mensagens de commit antigas que já citavam material não público ficam no
  histórico: reescrevê-lo exigiria *force push* no `main`, que a ADR 0003 proíbe.
- **Trade-offs aceitos:** explicar uma escolha sem a fonte original às vezes pede mais
  texto (por exemplo, "perfis de referência do projeto, conferidos por busca exaustiva"
  em vez de "os itens da lista").

## Diretriz de Conformidade de Código

- **Proibido:** criar pasta ou arquivo de spec por versão; citar, em código, teste, spec,
  CHANGELOG, README, texto da interface ou mensagem de commit, lista de exercício,
  solução, roteiro, slide ou outro material didático não publicado.
- **Obrigatório:** PR que muda comportamento atualiza a spec e o `CHANGELOG.md`; limitação
  declarada no README aponta para o critério ou o ticket que a trataria.

> Qualquer desvio desta regra viola as diretrizes de conformidade arquitetural do projeto
> e deve ser reportado para revisão antes de prosseguir.
