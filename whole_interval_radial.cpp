#include <boost/multiprecision/cpp_int.hpp>
#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <vector>

using Z=boost::multiprecision::int512_t;
static const Z S=Z(1)<<128, MASK=(Z(1)<<64)-1, SIGN=Z(1)<<191, MOD=Z(1)<<192;
static constexpr int N=17, JET=1+N+N*N, W=6;
struct I{Z lo,hi;};
struct J{I v,center;bool centered=false;std::array<I,N> g,cg;std::array<I,N*N> h;};
using V=std::array<J,3>;
static std::array<I,N> OFF;
static bool TIGHTEN_GRAD=true;
static Z rd(const uint64_t*p){Z x=p[2];x<<=64;x+=p[1];x<<=64;x+=p[0];if(x>=SIGN)x-=MOD;return x;}
static void wr(uint64_t*p,Z x){if(x<0)x+=MOD;for(int i=0;i<3;i++){p[i]=(x&MASK).convert_to<uint64_t>();x>>=64;}}
static I ri(const uint64_t*p){return{rd(p),rd(p+3)};}
static void wi(uint64_t*p,const I&a){wr(p,a.lo);wr(p+3,a.hi);}
static Z fd(const Z&a,const Z&b){Z q=a/b,r=a%b;if(r<0)--q;return q;}
static Z cd(const Z&a,const Z&b){Z q=a/b,r=a%b;if(r>0)++q;return q;}
static I zero(){return{0,0};}static I integer(int64_t n){Z x=n;x*=S;return{x,x};}
static I rat(int64_t n,int64_t d){Z x=Z(n)*S,y=d;return{fd(x,y),cd(x,y)};}
static I add(const I&a,const I&b){return{a.lo+b.lo,a.hi+b.hi};}
static I neg(const I&a){return{-a.hi,-a.lo};}static I sub(const I&a,const I&b){return{a.lo-b.hi,a.hi-b.lo};}
static I twice(const I&a){return{a.lo*2,a.hi*2};}
static I mul(const I&a,const I&b){Z p[4]={a.lo*b.lo,a.lo*b.hi,a.hi*b.lo,a.hi*b.hi};auto m=std::minmax_element(p,p+4);return{fd(*m.first,S),cd(*m.second,S)};}
static I sq(const I&a){Z x=a.lo*a.lo,y=a.hi*a.hi;return{a.lo<=0&&a.hi>=0?Z(0):fd(std::min(x,y),S),cd(std::max(x,y),S)};}
static I inv(const I&a){return{fd(S*S,a.hi),cd(S*S,a.lo)};}
static I root(const I&a){Z lo=(std::max(a.lo,Z(0))*S),hi=a.hi*S;Z l=boost::multiprecision::sqrt(lo),h=boost::multiprecision::sqrt(hi);if(h*h<hi)++h;return{l,h};}
static I divi(const I&a,int64_t n){return{fd(a.lo,Z(n)),cd(a.hi,Z(n))};}
static J con(I v){J a;a.v=v;a.center=v;for(auto&x:a.g)x=zero();for(auto&x:a.cg)x=zero();for(auto&x:a.h)x=zero();return a;}
static void intersect(I&x,const I&t){Z lo=std::max(x.lo,t.lo),hi=std::min(x.hi,t.hi);if(lo<=hi)x={lo,hi};}
static void narrow(J&r){if(!r.centered)return;I t=r.center;for(int i=0;i<N;i++)t=add(t,mul(r.g[i],OFF[i]));intersect(r.v,t);if(TIGHTEN_GRAD)for(int i=0;i<N;i++){I g=r.cg[i];for(int k=0;k<N;k++)g=add(g,mul(r.h[i*N+k],OFF[k]));intersect(r.g[i],g);}}
static void narrow_gradient(J&r){if(!r.centered)return;for(int i=0;i<N;i++){I g=r.cg[i];for(int k=0;k<N;k++)g=add(g,mul(r.h[i*N+k],OFF[k]));intersect(r.g[i],g);}}
static J add(const J&a,const J&b){J r;r.v=add(a.v,b.v);r.center=add(a.center,b.center);r.centered=a.centered||b.centered;for(int i=0;i<N;i++){r.g[i]=add(a.g[i],b.g[i]);r.cg[i]=add(a.cg[i],b.cg[i]);}for(int i=0;i<N*N;i++)r.h[i]=add(a.h[i],b.h[i]);narrow(r);return r;}
static J neg(const J&a){J r;r.v=neg(a.v);r.center=neg(a.center);r.centered=a.centered;for(int i=0;i<N;i++){r.g[i]=neg(a.g[i]);r.cg[i]=neg(a.cg[i]);}for(int i=0;i<N*N;i++)r.h[i]=neg(a.h[i]);narrow(r);return r;}
static J sub(const J&a,const J&b){return add(a,neg(b));}
static J mul(const J&a,const J&b){J r;r.v=mul(a.v,b.v);r.center=mul(a.center,b.center);r.centered=a.centered||b.centered;for(int i=0;i<N;i++){r.g[i]=add(mul(a.g[i],b.v),mul(a.v,b.g[i]));r.cg[i]=add(mul(a.cg[i],b.center),mul(a.center,b.cg[i]));}for(int i=0;i<N;i++)for(int j=0;j<N;j++){int k=i*N+j;r.h[k]=add(add(mul(a.h[k],b.v),mul(a.v,b.h[k])),add(mul(a.g[i],b.g[j]),mul(b.g[i],a.g[j])));}narrow(r);return r;}
static J square(const J&a){J r;r.v=sq(a.v);r.center=sq(a.center);r.centered=a.centered;for(int i=0;i<N;i++){r.g[i]=mul(mul(integer(2),a.v),a.g[i]);r.cg[i]=mul(mul(integer(2),a.center),a.cg[i]);}for(int i=0;i<N;i++)for(int j=0;j<N;j++){int k=i*N+j;r.h[k]=add(mul(mul(integer(2),a.v),a.h[k]),mul(integer(2),mul(a.g[i],a.g[j])));}narrow(r);return r;}
static J inverse(const J&a){I v=inv(a.v),fp=neg(sq(v)),fpp=mul(integer(2),mul(sq(v),v));I cv=inv(a.center),cfp=neg(sq(cv));J r;r.v=v;r.center=cv;r.centered=a.centered;for(int i=0;i<N;i++){r.g[i]=mul(a.g[i],fp);r.cg[i]=mul(a.cg[i],cfp);}for(int i=0;i<N;i++)for(int j=0;j<N;j++){int k=i*N+j;r.h[k]=add(mul(a.h[k],fp),mul(mul(a.g[i],a.g[j]),fpp));}narrow(r);return r;}
static J divide(const J&a,const J&b){return mul(a,inverse(b));}
static J sqrtj(const J&a){I v=root(a.v),fp=inv(mul(integer(2),v)),fpp=neg(inv(mul(integer(4),mul(a.v,v))));I cv=root(a.center),cfp=inv(mul(integer(2),cv));J r;r.v=v;r.center=cv;r.centered=a.centered;for(int i=0;i<N;i++){r.g[i]=mul(a.g[i],fp);r.cg[i]=mul(a.cg[i],cfp);}for(int i=0;i<N;i++)for(int j=0;j<N;j++){int k=i*N+j;r.h[k]=add(mul(a.h[k],fp),mul(mul(a.g[i],a.g[j]),fpp));}narrow(r);return r;}
static J scale(const J&a,int64_t n,int64_t d=1){I c=rat(n,d);J r;r.v=mul(a.v,c);r.center=mul(a.center,c);r.centered=a.centered;for(int i=0;i<N;i++){r.g[i]=mul(a.g[i],c);r.cg[i]=mul(a.cg[i],c);}for(int i=0;i<N*N;i++)r.h[i]=mul(a.h[i],c);narrow(r);return r;}
static V add(const V&a,const V&b){return{::add(a[0],b[0]),::add(a[1],b[1]),::add(a[2],b[2])};}
static V sub(const V&a,const V&b){return{::sub(a[0],b[0]),::sub(a[1],b[1]),::sub(a[2],b[2])};}
static V scale(const V&a,const J&s){return{mul(a[0],s),mul(a[1],s),mul(a[2],s)};}
static V cross(const V&a,const V&b){return sub(V{mul(a[1],b[2]),mul(a[2],b[0]),mul(a[0],b[1])},V{mul(a[2],b[1]),mul(a[0],b[2]),mul(a[1],b[0])});}
static J dot(const V&a,const V&b){return add(add(square(a[0]),square(a[1])),square(a[2]));}
static std::vector<V> geometry(const std::vector<I>&q,const std::vector<I>&qc,const uint8_t*charts,bool variables){
    J b=sqrtj(con(integer(8))),r=sqrtj(con(integer(3)));std::vector<V>p(N+3);
    p[0]={con(zero()),con(zero()),con(zero())};p[1]={con(integer(1)),con(zero()),con(zero())};
    p[2]={con(rat(4,3)),scale(b,1,3),con(zero())};
    for(int z=3;z<N+3;z++){
        J t=con(q[z-3]);if(variables){t.g[z-3]=integer(1);t.cg[z-3]=integer(1);t.centered=true;t.center=qc[z-3];}
        J t2=square(t),den=add(con(integer(1)),t2);
        J c=charts[z-3]?divide(sub(t2,con(integer(1))),den):divide(sub(con(integer(1)),t2),den);
        J s=divide(scale(t,2),den);V e=sub(p[z-1],p[z-2]),u=sub(p[z-2],p[z-3]);
        V normal=scale(cross(u,e),scale(b,3,8)),tangent=cross(normal,e);
        p[z]=add(p[z-1],add(scale(e,con(rat(1,3))),
            scale(add(scale(tangent,c),scale(normal,s)),scale(b,1,3))));
    }
    return p;
}
static J centroid(const std::array<V,4>&sites,int mask,int axis){J sum=con(zero());int count=0;for(int v=0;v<4;v++)if(mask&(1<<v)){sum=add(sum,sites[v][axis]);count++;}return scale(sum,1,count);}
extern "C" void whole_interval_energy(const uint64_t* box,const uint64_t* center,
        const uint8_t* charts,const uint32_t* keys,const uint64_t* coefficients,
        size_t count,uint64_t* output){
    TIGHTEN_GRAD=true;
    std::vector<I>q(N),qc(N);for(int i=0;i<N;i++){q[i]=ri(box+i*W);qc[i]=ri(center+i*W);}
    for(int i=0;i<N;i++)OFF[i]=sub(q[i],qc[i]);
    auto p=geometry(q,qc,charts,true),pc=geometry(qc,qc,charts,false);
    std::vector<std::array<V,4>>sites(N+3),centersites(N+3);
    J bend=scale(sqrtj(con(integer(8))),3,8),radius=scale(sqrtj(con(integer(3))),1,3);
    for(int i=0;i<N+3;i++){
        int a=i>=2?i-2:0,b=i>=2?i-1:1,c=i>=2?i:2;
        V e=sub(p[c],p[b]),u=sub(p[b],p[a]),normal=scale(cross(u,e),bend),tangent=cross(normal,e);
        V ec=sub(pc[c],pc[b]),uc=sub(pc[b],pc[a]),normalc=scale(cross(uc,ec),bend),tangentc=cross(normalc,ec);
        for(int v=0;v<4;v++){
            int x=1-2*(v&1),y=1-2*(v>>1);
            sites[i][v]=add(p[i],scale(add(scale(e,con(integer(x))),add(scale(tangent,con(integer(y))),scale(normal,con(integer(x*y))))),radius));
            centersites[i][v]=add(pc[i],scale(add(scale(ec,con(integer(x))),add(scale(tangentc,con(integer(y))),scale(normalc,con(integer(x*y))))),radius));
        }
    }
    // Center and narrow the recursive carrier once. Per-response distance
    // derivatives then use these retained enclosures without repeating a
    // 17x17 Taylor-gradient contraction for every response.
    TIGHTEN_GRAD=false;
    J total=con(zero());
    for(size_t z=0;z<count;z++){
        size_t alternatives=keys[z*4]>>16;if(!alternatives)alternatives=1;
        int i=keys[z*4]&65535,a=keys[z*4+1],j=keys[z*4+2],b=keys[z*4+3];
        V dl,anchor;for(int axis=0;axis<3;axis++){
            dl[axis]=sub(centroid(sites[i],a,axis),centroid(sites[j],b,axis));
            anchor[axis]=sub(centroid(centersites[i],a,axis),centroid(centersites[j],b,axis));
        }
        // Tighten each distance coordinate by the center value plus its
        // first-order displacement over the same source-bound parameter box.
        for(int axis=0;axis<3;axis++){
            I value=anchor[axis].v;
            for(int k=0;k<N;k++)value=add(value,mul(dl[axis].g[k],sub(q[k],qc[k])));
            dl[axis].v=value;
        }
        J d=sqrtj(dot(dl,dl));narrow_gradient(d);J enclosed=con(zero());
        for(size_t option=0;option<alternatives;option++){
        const uint64_t*co=coefficients+(z+option)*3*W;
        J term=con(zero());
        I stiffness=ri(co),first=ri(co+W),second=ri(co+2*W);
        I slope=sub(mul(stiffness,d.v),first);
        term.v=divi(add(sub(mul(stiffness,sq(d.v)),mul(twice(first),d.v)),second),2);
        for(int i=0;i<N;i++)term.g[i]=mul(d.g[i],slope);
        for(int i=0;i<N;i++)for(int j=0;j<N;j++){
            int k=i*N+j;I h=add(mul(d.h[k],slope),mul(mul(d.g[i],d.g[j]),stiffness));
            term.h[k]=h;
        }
        if(!option)enclosed=term;
        else{
            auto hull=[](I a,I b)->I{return{std::min(a.lo,b.lo),std::max(a.hi,b.hi)};};
            enclosed.v=hull(enclosed.v,term.v);
            for(int k=0;k<N;k++)enclosed.g[k]=hull(enclosed.g[k],term.g[k]);
            for(int k=0;k<N*N;k++)enclosed.h[k]=hull(enclosed.h[k],term.h[k]);
        }
        }
        total.v=add(total.v,enclosed.v);
        for(int k=0;k<N;k++)total.g[k]=add(total.g[k],enclosed.g[k]);
        for(int k=0;k<N*N;k++)total.h[k]=add(total.h[k],enclosed.h[k]);
        z+=alternatives-1;
    }
    wi(output,total.v);for(int i=0;i<N;i++)wi(output+(1+i)*W,total.g[i]);
    for(int i=0;i<N*N;i++)wi(output+(1+N+i)*W,total.h[i]);
}
