    6b42:	55                   	push   bp
    6b43:	8b ec                	mov    bp,sp
    6b45:	83 ec 0a             	sub    sp,0xa
    6b48:	56                   	push   si
    6b49:	57                   	push   di
    6b4a:	8b 7e 06             	mov    di,WORD PTR ss:[bp+0x6]
    6b4d:	8b df                	mov    bx,di
    6b4f:	c1 e3 02             	shl    bx,0x2
    6b52:	8b 87 46 09          	mov    ax,WORD PTR ds:[bx+0x946]
    6b56:	8b 97 44 09          	mov    dx,WORD PTR ds:[bx+0x944]
    6b5a:	89 46 fc             	mov    WORD PTR ss:[bp-0x4],ax
    6b5d:	89 56 fa             	mov    WORD PTR ss:[bp-0x6],dx
    6b60:	33 f6                	xor    si,si
    6b62:	eb 2f                	jmp    0x6b93
    6b64:	c4 5e fa             	les    bx,DWORD PTR ss:[bp-0x6]
    6b67:	26 f7 07 40 00       	test   WORD PTR es:[bx],0x40
    6b6c:	74 04                	je     0x6b72
    6b6e:	26 83 37 50          	xor    WORD PTR es:[bx],0x50
    6b72:	c4 5e fa             	les    bx,DWORD PTR ss:[bp-0x6]
    6b75:	26 8b 07             	mov    ax,WORD PTR es:[bx]
    6b78:	89 46 fe             	mov    WORD PTR ss:[bp-0x2],ax
    6b7b:	f7 46 fe 14 00       	test   WORD PTR ss:[bp-0x2],0x14
    6b80:	75 0c                	jne    0x6b8e
    6b82:	ff 76 fc             	push   WORD PTR ss:[bp-0x4]
    6b85:	53                   	push   bx
    6b86:	9a 96 04 81 05       	callf  0x581:0x496
    6b8b:	83 c4 04             	add    sp,0x4
    6b8e:	83 46 fa 20          	add    WORD PTR ss:[bp-0x6],0x20
    6b92:	46                   	inc    si
    6b93:	8b df                	mov    bx,di
    6b95:	d1 e3                	shl    bx,0x1
    6b97:	39 b7 c8 08          	cmp    WORD PTR ds:[bx+0x8c8],si
    6b9b:	7f c7                	jg     0x6b64
    6b9d:	83 ff 03             	cmp    di,0x3
    6ba0:	75 5f                	jne    0x6c01
    6ba2:	8b 1e c2 08          	mov    bx,WORD PTR ds:0x8c2
    6ba6:	c1 e3 02             	shl    bx,0x2
    6ba9:	c4 9f 04 7a          	les    bx,DWORD PTR ds:[bx+0x7a04]
    6bad:	26 ff b7 a2 00       	push   WORD PTR es:[bx+0xa2]
    6bb2:	26 ff b7 a0 00       	push   WORD PTR es:[bx+0xa0]
    6bb7:	9a 0f 00 c0 09       	callf  0x9c0:0xf
    6bbc:	83 c4 04             	add    sp,0x4
    6bbf:	89 56 f8             	mov    WORD PTR ss:[bp-0x8],dx
    6bc2:	89 46 f6             	mov    WORD PTR ss:[bp-0xa],ax
    6bc5:	33 f6                	xor    si,si
    6bc7:	3b 36 64 09          	cmp    si,WORD PTR ds:0x964
    6bcb:	7d 34                	jge    0x6c01
    6bcd:	8b 46 f8             	mov    ax,WORD PTR ss:[bp-0x8]
    6bd0:	8b 56 f6             	mov    dx,WORD PTR ss:[bp-0xa]
    6bd3:	83 c2 0c             	add    dx,0xc
    6bd6:	89 46 fc             	mov    WORD PTR ss:[bp-0x4],ax
    6bd9:	89 56 fa             	mov    WORD PTR ss:[bp-0x6],dx
    6bdc:	50                   	push   ax
    6bdd:	52                   	push   dx
    6bde:	9a 96 04 81 05       	callf  0x581:0x496
    6be3:	83 c4 04             	add    sp,0x4
    6be6:	46                   	inc    si
    6be7:	ff 76 f8             	push   WORD PTR ss:[bp-0x8]
    6bea:	ff 76 f6             	push   WORD PTR ss:[bp-0xa]
    6bed:	9a 2e 00 c0 09       	callf  0x9c0:0x2e
    6bf2:	83 c4 04             	add    sp,0x4
    6bf5:	89 56 f8             	mov    WORD PTR ss:[bp-0x8],dx
    6bf8:	89 46 f6             	mov    WORD PTR ss:[bp-0xa],ax
    6bfb:	3b 36 64 09          	cmp    si,WORD PTR ds:0x964
    6bff:	7c cc                	jl     0x6bcd
    6c01:	5f                   	pop    di
    6c02:	5e                   	pop    si
    6c03:	c9                   	leave
    6c04:	cb                   	retf
