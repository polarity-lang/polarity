
if exists("b:current_syntax")
  finish
endif

syn keyword xfuncControl data codata impl def codef match comatch absurd Type

syn match xfuncSyntax '\v%(;|\=\>|,|:|\.)'

syn match xfuncComment '--.*$'

syn match xfuncConstructor '\v<[A-Z]\k*>'

syn match xfuncDestructorCall '\v\.\k+>' contains=xfuncSyntax

hi def link xfuncControl Keyword
hi def link xfuncSyntax Macro
hi def link xfuncComment Comment
hi def link xfuncConstructor Statement
hi def link xfuncDestructorCall Type


