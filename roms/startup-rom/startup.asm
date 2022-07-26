; Forked from http://hp.vector.co.jp/authors/VA042397/nes/sample.html

.setcpu		"6502"
.autoimport	on

; iNESヘッダ
.segment "HEADER"
	.byte	$4E, $45, $53, $1A	; "NES" Header
	.byte	$02			; PRG-BANKS
	.byte	$01			; CHR-BANKS
	.byte	$01			; Vetrical Mirror
	.byte	$00			; 
	.byte	$00, $00, $00, $00	; 
	.byte	$00, $00, $00, $00	; 

.segment "STARTUP"
; リセット割り込み
.proc	Reset
	sei
	ldx	#$ff
	txs

; スクリーンオフ
	lda	#$00
	sta	$2000
	sta	$2001

; パレットテーブルへ転送(BG用のみ転送)
	lda	#$3f
	sta	$2006
	lda	#$00
	sta	$2006
	ldx	#$00
	ldy	#$10
copypal:
	lda	palettes, x
	sta	$2007
	inx
	dey
	bne	copypal

; ネームテーブルへ転送("WELCOME TO NESTY!")
	lda	#$21
	sta	$2006
	lda	#$87
	sta	$2006
	ldx	#$00
	ldy	#$11		; 17文字表示
copymap1:
	lda	welcome, x
	sta	$2007
	inx
	dey
	bne	copymap1

; ネームテーブルへ転送("PLEASE LOAD A ROM FILE TO START")
	lda	#$21
	sta	$2006
	lda	#$E2
	sta	$2006
	ldx	#$00
	ldy	#$1B		; 27文字表示
copymap2:
	lda	instructions, x
	sta	$2007
	inx
	dey
	bne	copymap2

; スクロール設定
	lda	#$00
	sta	$2005
	sta	$2005

; スクリーンオン
	lda	#$08
	sta	$2000
	lda	#$1e
	sta	$2001

; 無限ループ
mainloop:
	jmp	mainloop
.endproc

; パレットテーブル
palettes:
	.byte	$0f, $00, $10, $20
	.byte	$0f, $06, $16, $26
	.byte	$0f, $08, $18, $28
	.byte	$0f, $0a, $1a, $2a

; 表示文字列
welcome:
	.byte	"WELCOME TO NESTY!"
instructions:
	.byte	"LOAD A ROM FILE TO CONTINUE"

.segment "VECINFO"
	.word	$0000
	.word	Reset
	.word	$0000

; パターンテーブル
.segment "CHARS"
	.incbin	"character.chr"
