; Forked from http://hp.vector.co.jp/authors/VA042397/nes/sample.html

.setcpu		"6502"
.autoimport	on

; iNES�w�b�_
.segment "HEADER"
	.byte	$4E, $45, $53, $1A	; "NES" Header
	.byte	$02			; PRG-BANKS
	.byte	$01			; CHR-BANKS
	.byte	$01			; Vetrical Mirror
	.byte	$00			; 
	.byte	$00, $00, $00, $00	; 
	.byte	$00, $00, $00, $00	; 

.segment "STARTUP"
; ���Z�b�g���荞��
.proc	Reset
	sei
	ldx	#$ff
	txs

; �X�N���[���I�t
	lda	#$00
	sta	$2000
	sta	$2001

; �p���b�g�e�[�u���֓]��(BG�p�̂ݓ]��)
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

; �l�[���e�[�u���֓]��("WELCOME TO NESTY!")
	lda	#$21
	sta	$2006
	lda	#$87
	sta	$2006
	ldx	#$00
	ldy	#$11		; 17�����\��
copymap1:
	lda	welcome, x
	sta	$2007
	inx
	dey
	bne	copymap1

; �l�[���e�[�u���֓]��("PLEASE LOAD A ROM FILE TO START")
	lda	#$21
	sta	$2006
	lda	#$E2
	sta	$2006
	ldx	#$00
	ldy	#$1B		; 27�����\��
copymap2:
	lda	instructions, x
	sta	$2007
	inx
	dey
	bne	copymap2

; �X�N���[���ݒ�
	lda	#$00
	sta	$2005
	sta	$2005

; �X�N���[���I��
	lda	#$08
	sta	$2000
	lda	#$1e
	sta	$2001

; �������[�v
mainloop:
	jmp	mainloop
.endproc

; �p���b�g�e�[�u��
palettes:
	.byte	$0f, $00, $10, $20
	.byte	$0f, $06, $16, $26
	.byte	$0f, $08, $18, $28
	.byte	$0f, $0a, $1a, $2a

; �\��������
welcome:
	.byte	"WELCOME TO NESTY!"
instructions:
	.byte	"LOAD A ROM FILE TO CONTINUE"

.segment "VECINFO"
	.word	$0000
	.word	Reset
	.word	$0000

; �p�^�[���e�[�u��
.segment "CHARS"
	.incbin	"character.chr"
